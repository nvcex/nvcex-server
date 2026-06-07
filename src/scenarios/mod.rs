pub mod basic;
#[path ="六花とつむぎのスタンプラリー.rs"]
mod 六花とつむぎのスタンプラリー;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use crate::{scenarios::{basic::default_render_guidance, 六花とつむぎのスタンプラリー::六花とつむぎのスタンプラリーScenario}, voices::{SpeechText, Voice}};

pub struct Input<'a> {
    pub(crate) text: &'a String,
    pub(crate) guidance: Option<&'a crate::pathfinder4::Guidance>,
}

/// Scenario trait: implementors can render a `Guidance` into a text string.
pub trait Scenario {
    fn name(&self) -> String;
    fn render(&self, input: Input) -> SpeechText;
}

pub type SharedScenario = Arc<dyn Scenario + Send + Sync>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VoicevoxScenario {
    pub speaker_name: String,
    pub style_name: String,
    pub style_id: u32,
    pub use_parser: bool,
}

impl Scenario for VoicevoxScenario {
    fn render(&self, input: Input) -> SpeechText {
        if self.use_parser {
            if let Some(guidance) = input.guidance {
                let s = default_render_guidance(guidance).unwrap();
                SpeechText::DynamicText(s, Arc::new(Voice::VOICEVOX(self.style_id)))
            } else {
                SpeechText::DynamicText(input.text.clone(), Arc::new(Voice::VOICEVOX(self.style_id)))
            }
        } else {
            SpeechText::DynamicText(input.text.clone(), Arc::new(Voice::VOICEVOX(self.style_id)))
        }
    }

    fn name(&self) -> String {
        format!("VOICEVOX/{}/{}{}", self.speaker_name, self.style_name, if self.use_parser { "/P" } else {""})
    }
}

pub fn build_scenarios(speakers: &[crate::voices::Speaker]) -> HashMap<String, SharedScenario> {
    let mut scenarios = HashMap::new();

    let mut add = |scenario: Arc<dyn Scenario + Send + Sync>| scenarios.insert(scenario.name(), scenario);
    for speaker in speakers {
        for style in &speaker.styles {
            let scenario = VoicevoxScenario {
                speaker_name: speaker.name.clone(),
                style_name: style.name.clone(),
                style_id: style.id,
                use_parser: false,
            };
            add(Arc::new(scenario));
        }
    }

    for speaker in speakers {
        for style in &speaker.styles {
            let scenario = VoicevoxScenario {
                speaker_name: speaker.name.clone(),
                style_name: style.name.clone(),
                style_id: style.id,
                use_parser: true,
            };
            add(Arc::new(scenario));
        }
    }

    add(Arc::new(六花とつむぎのスタンプラリーScenario {}));

    scenarios
}
