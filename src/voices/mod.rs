

pub mod voicevox;

pub use voicevox::{Speaker, SpeakerStyle, VoicevoxClient};

use async_recursion::async_recursion;
use sha2::Sha256;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};


#[derive(Debug)]
pub struct StaticVoice {
    scenario_name: String,
    name: String
}

impl StaticVoice {
    pub fn new(scenario_name: impl Into<String>, name: impl Into<String>) -> Self {
        Self { scenario_name: scenario_name.into(), name: name.into() }
    }
}

#[derive(Debug)]
pub enum Voice {
    VOICEVOX(Speaker, SpeakerStyle),
    Static(StaticVoice),
}

impl Voice {
    pub fn name(&self) -> String {
        match self {
            Voice::VOICEVOX(speaker, style) => format!("VOICEVOX {} ({})", speaker.name, style.name),
            Voice::Static(v) => format!("{} ({})", v.name, v.scenario_name),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SpeechText {
    StaticText(String, Arc<Voice>),
    DynamicText(String, Arc<Voice>),
    Seq(Vec<SpeechText>),
}

pub fn format_speech_text(text: &SpeechText) -> String {
    fn fmt(text: &SpeechText, indent: usize) -> String {
        let pad = "  ".repeat(indent);
        match text {
            SpeechText::StaticText(t, voice) => format!("{}static {:?} [{}]", pad, t, voice.name()),
            SpeechText::DynamicText(t, voice) => format!("{}dynamic {:?} [{}]", pad, t, voice.name()),
            SpeechText::Seq(children) => children.iter()
                .map(|c| fmt(c, indent))
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
    fmt(text, 0)
}

#[derive(Debug)]
pub enum VoiceError {
    Request(reqwest::Error),
    Decode(String),
    NotFound,
    Unsupported,
}

impl std::fmt::Display for VoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoiceError::Request(err) => write!(f, "request failed: {}", err),
            VoiceError::Decode(err) => write!(f, "audio decode failed: {}", err),
            VoiceError::NotFound => write!(f, "audio file not found"),
            VoiceError::Unsupported => write!(f, "unsupported voice type"),
        }
    }
}

impl std::error::Error for VoiceError {}

impl From<reqwest::Error> for VoiceError {
    fn from(err: reqwest::Error) -> Self {
        VoiceError::Request(err)
    }
}

#[derive(Clone, Debug)]
pub struct StaticVoiceRepository {
    root: PathBuf,
}

impl StaticVoiceRepository {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn file_path(&self, voice: &StaticVoice, text: &str) -> PathBuf {
        self.root
            .join(&voice.scenario_name)
            .join(&voice.name)
            .join(format!("{}.wav", text))
    }

    pub async fn get(&self, voice: &StaticVoice, text: &str) -> Result<bytes::Bytes, VoiceError> {
        let path = self.file_path(voice, text);
        tokio::fs::read(&path).await
            .map(bytes::Bytes::from)
            .map_err(|_| VoiceError::NotFound)
    }
}

#[derive(Clone, Debug)]
pub struct VoiceProviders {
    pub static_voices: StaticVoiceRepository,
    pub voicebox: Option<VoicevoxClient>
}

impl VoiceProviders {
    pub async fn static_text(&self, text: &str, voice: &Voice) -> Result<bytes::Bytes, VoiceError> {
        match voice {
            Voice::VOICEVOX(_, style) => self.try_voicevox(text, style.id).await,
            Voice::Static(v) => self.static_voices.get(v, text).await,
        }
    }

    pub async fn dynamic_text(&self, text: &str, voice: &Voice) -> Result<bytes::Bytes, VoiceError> {
        match voice {
            Voice::VOICEVOX(_, style) => self.try_voicevox(text, style.id).await,
            Voice::Static(_) => Err(VoiceError::Unsupported),
        }
    }

    pub async fn try_voicevox(&self, text: &str, voice_id: u32) -> Result<bytes::Bytes, VoiceError> {
        let client = self.voicebox.as_ref().ok_or(VoiceError::Unsupported)?;
        Ok(client.synthesize(text, voice_id).await?)
    }
}

pub async fn text_to_speech(text: SpeechText, provider: &VoiceProviders) -> Result<Vec<u8>, VoiceError> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: 24_000,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    let mut writer = WavWriter::new(&mut cursor, spec)
        .map_err(|err| VoiceError::Decode(err.to_string()))?;

    write_speech_text(&mut writer, &text, provider).await?;

    writer.finalize().map_err(|err| VoiceError::Decode(err.to_string()))?;
    Ok(cursor.into_inner())
}

#[async_recursion]
async fn write_speech_text<W: std::io::Write + std::io::Seek + Send>(
    writer: &mut WavWriter<W>,
    text: &SpeechText,
    provider: &VoiceProviders
) -> Result<(), VoiceError> {
    match text {
        SpeechText::StaticText(text, voice) => {
            let segment_bytes = provider.static_text(text, voice).await?;
            write_segment(writer, segment_bytes).await
        }
        SpeechText::DynamicText(text, voice) => {
            let segment_bytes = provider.dynamic_text(text, voice).await?;
            write_segment(writer, segment_bytes).await
        }
        SpeechText::Seq(children) => {
            for child in children {
                write_speech_text(writer, child, provider).await?;
            }
            Ok(())
        }
    }
}

async fn write_segment<W: std::io::Write + std::io::Seek + Send>(
    writer: &mut WavWriter<W>,
    bytes: bytes::Bytes,
) -> Result<(), VoiceError> {
    let cursor = Cursor::new(bytes);
    let mut reader = WavReader::new(cursor)
        .map_err(|e| VoiceError::Decode(e.to_string()))?;

    let spec = reader.spec();
    if spec.channels != 1 || spec.bits_per_sample != 16 {
        return Err(VoiceError::Decode(format!(
            "expected mono 16-bit WAV, got {}ch {}bit", spec.channels, spec.bits_per_sample
        )));
    }

    let samples: Vec<i16> = reader.samples::<i16>()
        .collect::<Result<_, _>>()
        .map_err(|e| VoiceError::Decode(e.to_string()))?;

    let samples = if spec.sample_rate != 24_000 {
        resample(samples, spec.sample_rate, 24_000)?
    } else {
        samples
    };

    for s in samples {
        writer.write_sample(s).map_err(|e| VoiceError::Decode(e.to_string()))?;
    }

    Ok(())
}

fn resample(samples: Vec<i16>, from_rate: u32, to_rate: u32) -> Result<Vec<i16>, VoiceError> {
    use rubato::{FastFixedIn, PolynomialDegree, Resampler};

    let ratio = to_rate as f64 / from_rate as f64;
    let input: Vec<f64> = samples.iter().map(|&s| s as f64 / 32768.0).collect();

    const CHUNK: usize = 1024;
    let mut resampler = FastFixedIn::<f64>::new(ratio, 2.0, PolynomialDegree::Septic, CHUNK, 1)
        .map_err(|e| VoiceError::Decode(e.to_string()))?;

    let mut out: Vec<f64> = Vec::new();
    let mut pos = 0;

    while pos + CHUNK <= input.len() {
        let chunk = input[pos..pos + CHUNK].to_vec();
        let result = resampler.process(&[chunk], None)
            .map_err(|e| VoiceError::Decode(e.to_string()))?;
        out.extend_from_slice(&result[0]);
        pos += CHUNK;
    }

    if pos < input.len() {
        let remaining = input.len() - pos;
        let expected_out = (remaining as f64 * ratio).ceil() as usize;
        let mut tail = input[pos..].to_vec();
        tail.resize(CHUNK, 0.0);
        let result = resampler.process(&[tail], None)
            .map_err(|e| VoiceError::Decode(e.to_string()))?;
        out.extend_from_slice(&result[0][..expected_out.min(result[0].len())]);
    }

    Ok(out.iter().map(|&s| (s * 32768.0).clamp(-32768.0, 32767.0) as i16).collect())
}

