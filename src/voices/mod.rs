

pub mod voicevox;

pub use voicevox::{Speaker, VoicevoxClient};

use std::future::Future;
use std::io::Cursor;
use std::pin::Pin;
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Voice {
    VOICEVOX(u32),
}

impl Voice {
    fn id(&self) -> u32 {
        match self {
            Voice::VOICEVOX(id) => *id,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SpeechText {
    StaticText(String, Voice),
    DynamicText(String, Voice),
    Seq(Vec<SpeechText>),
}

#[derive(Debug)]
pub enum VoiceError {
    Request(reqwest::Error),
    Decode(String),
}

impl std::fmt::Display for VoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoiceError::Request(err) => write!(f, "request failed: {}", err),
            VoiceError::Decode(err) => write!(f, "audio decode failed: {}", err),
        }
    }
}

impl std::error::Error for VoiceError {}

impl From<reqwest::Error> for VoiceError {
    fn from(err: reqwest::Error) -> Self {
        VoiceError::Request(err)
    }
}

pub async fn text_to_speech(text: SpeechText, voicebox: &VoicevoxClient) -> Result<Vec<u8>, VoiceError> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    let mut writer = WavWriter::new(&mut cursor, spec)
        .map_err(|err| VoiceError::Decode(err.to_string()))?;

    write_speech_text(&mut writer, &text, voicebox).await?;

    writer.finalize().map_err(|err| VoiceError::Decode(err.to_string()))?;
    Ok(cursor.into_inner())
}

fn write_speech_text<'a, W: std::io::Write + std::io::Seek + Send + 'a>(
    writer: &'a mut WavWriter<W>,
    text: &'a SpeechText,
    voicebox: &'a VoicevoxClient,
) -> Pin<Box<dyn Future<Output = Result<(), VoiceError>> + Send + 'a>> {
    Box::pin(async move {
        match text {
            SpeechText::StaticText(content, voice) | SpeechText::DynamicText(content, voice) => {
                write_segment(writer, content, *voice, voicebox).await
            }
            SpeechText::Seq(children) => {
                for child in children {
                    write_speech_text(writer, child, voicebox).await?;
                }
                Ok(())
            }
        }
    })
}

async fn write_segment<W: std::io::Write + std::io::Seek + Send>(
    writer: &mut WavWriter<W>,
    content: &str,
    voice: Voice,
    voicebox: &VoicevoxClient,
) -> Result<(), VoiceError> {
    let segment_bytes = voicebox.synthesize(content, voice.id()).await?;
    let cursor = Cursor::new(segment_bytes);
    let mut reader = WavReader::new(cursor)
        .map_err(|err| VoiceError::Decode(err.to_string()))?;

    let spec = reader.spec();
    if spec.channels != 1 || spec.sample_rate != 16_000 || spec.bits_per_sample != 16 {
        return Err(VoiceError::Decode(
            "expected 16kHz mono 16-bit WAV from Voicevox".to_string(),
        ));
    }

    for sample in reader.samples::<i16>() {
        let sample = sample.map_err(|err| VoiceError::Decode(err.to_string()))?;
        writer.write_sample(sample).map_err(|err| VoiceError::Decode(err.to_string()))?;
    }

    Ok(())
}

