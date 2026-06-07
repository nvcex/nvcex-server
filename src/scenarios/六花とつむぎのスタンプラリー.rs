use std::sync::Arc;
use crate::{pathfinder4::Guidance, scenarios::Scenario, voices::{SpeechText, StaticVoice, Voice}};

const SCENARIO: &str = "六花とつむぎのスタンプラリー";

#[derive(Debug, Clone)]
pub struct 六花とつむぎのスタンプラリーScenario {
    pub 六花: Arc<Voice>,
    pub つむぎ: Arc<Voice>,
    pub 六花とつむぎ: Arc<Voice>,
}

impl Scenario for 六花とつむぎのスタンプラリーScenario {
    fn name(&self) -> String {
        SCENARIO.to_string()
    }

    fn render(&self, input: super::Input) -> SpeechText {
        use rand::{SeedableRng, rngs::StdRng};
        let mut rng = StdRng::seed_from_u64(input.seed);
        input.guidance
            .map(|g| self.render_guidance(g, &mut rng))
            .unwrap_or(SpeechText::DynamicText(input.text.clone(), self.つむぎ.clone()))
    }

}

impl 六花とつむぎのスタンプラリーScenario {
    pub fn new(つむぎ: Arc<Voice>) -> Self {
        Self {
            六花: Arc::new(Voice::Static(StaticVoice::new(SCENARIO, "六花"))),
            つむぎ,
            六花とつむぎ: Arc::new(Voice::Static(StaticVoice::new(SCENARIO, "六花とつむぎ"))),
        }
    }

    fn render_guidance(&self, g: &Guidance, rng: &mut rand::rngs::StdRng) -> SpeechText {
        todo!()
    }
}
