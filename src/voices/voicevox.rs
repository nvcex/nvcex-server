

use bytes::Bytes;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone)]
pub struct VoicevoxClient {
    client: Client,
    base_url: Url,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Speaker {
    pub name: String,
    pub speaker_uuid: String,
    pub styles: Vec<SpeakerStyle>,
    pub version: String,
    pub supported_features: SupportedFeatures,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpeakerStyle {
    pub name: String,
    pub id: u32,
    #[serde(rename = "type")]
    pub style_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupportedFeatures {
    pub permitted_synthesis_morphing: String,
}

impl VoicevoxClient {
    pub fn new(base_url: impl AsRef<str>) -> Self {
        VoicevoxClient {
            client: Client::new(),
            base_url: Url::parse(base_url.as_ref()).expect("invalid Voicevox base URL"),
        }
    }

    pub async fn speakers(&self) -> Result<Vec<Speaker>, reqwest::Error> {
        let url = self.base_url.join("speakers").expect("invalid speakers URL");
        let response = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?;

        response.json::<Vec<Speaker>>().await
    }

    pub async fn audio_query(&self, text: &str, speaker: u32) -> Result<Bytes, reqwest::Error> {
        let url = self.base_url.join("audio_query").expect("invalid audio_query URL");
        let speaker_s = speaker.to_string();

        let response = self
            .client
            .post(url)
            .query(&[
                ("text", text),
                ("speaker", speaker_s.as_str()),
                ("enable_katakana_english", "true"),
            ])
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?;

        response.bytes().await
    }

    pub async fn synthesis(&self, query_body: Bytes, speaker: u32) -> Result<Bytes, reqwest::Error> {
        let url = self.base_url.join("synthesis").expect("invalid synthesis URL");
        let speaker_s = speaker.to_string();

        let response = self
            .client
            .post(url)
            .query(&[
                ("speaker", speaker_s.as_str()),
                ("enable_interrogative_upspeak", "true"),
            ])
            .header("Content-Type", "application/json")
            .body(query_body)
            .send()
            .await?
            .error_for_status()?;

        response.bytes().await
    }

    pub async fn synthesize(&self, text: &str, speaker: u32) -> Result<Bytes, reqwest::Error> {
        let audio_query_body = self.audio_query(text, speaker).await?;
        self.synthesis(audio_query_body, speaker).await
    }
}

