pub mod basic;
#[path ="六花とつむぎのスタンプラリー.rs"]
mod 六花とつむぎのスタンプラリー;

use std::collections::HashMap;
use std::sync::Arc;
use crate::{canned_message::{Canned, default_canned_message}, scenarios::{basic::default_render_guidance, 六花とつむぎのスタンプラリー::六花とつむぎのスタンプラリーScenario}, voices::{SpeechText, Voice}};

pub struct Input<'a> {
    pub(crate) text: &'a String,
    pub(crate) guidance: Option<&'a crate::pathfinder4::Guidance>,
    pub(crate) seed: u64,
}

impl<'a> Input<'a> {
    pub fn new(text: &'a String, guidance: Option<&'a crate::pathfinder4::Guidance>, data: &str) -> Self {
        use sha2::{Sha256, Digest};
        let hash = Sha256::digest(data.as_bytes());
        let seed = u64::from_le_bytes(hash[..8].try_into().unwrap());
        Self { text, guidance, seed }
    }
}

pub trait Scenario {
    fn name(&self) -> String;
    fn render(&self, input: Input) -> SpeechText;
    fn render_canned_message(&self, c: Canned) -> SpeechText;
}

pub type SharedScenario = Arc<dyn Scenario + Send + Sync>;

#[derive(Debug, Clone)]
pub struct VoicevoxScenario {
    pub voice: Arc<Voice>,
    pub use_parser: bool,
}

impl Scenario for VoicevoxScenario {
    fn render(&self, input: Input) -> SpeechText {
        let text = if self.use_parser {
            input.guidance
                .and_then(|g| default_render_guidance(g).ok())
                .unwrap_or_else(|| input.text.clone())
        } else {
            input.text.clone()
        };
        SpeechText::DynamicText(text, self.voice.clone())
    }

    fn render_canned_message(&self, c: Canned) -> SpeechText {
        SpeechText::DynamicText(default_canned_message(c), self.voice.clone())
    }

    fn name(&self) -> String {
        format!("{}{}", self.voice.name(), if self.use_parser { "/P" } else { "" })
    }
}

pub fn build_scenarios(speakers: &[crate::voices::Speaker]) -> HashMap<String, SharedScenario> {
    let mut scenarios = HashMap::new();
    let mut add = |scenario: Arc<dyn Scenario + Send + Sync>| scenarios.insert(scenario.name(), scenario);

    for speaker in speakers {
        for style in &speaker.styles {
            let voice = Arc::new(Voice::VOICEVOX(speaker.clone(), style.clone()));
            for use_parser in [false, true] {
                add(Arc::new(VoicevoxScenario { voice: voice.clone(), use_parser }));
            }
            if style.id == 8 {
                add(Arc::new(六花とつむぎのスタンプラリーScenario::new(voice.clone(), )));
            }
        }
    }

    scenarios
}
