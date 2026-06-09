use std::{sync::Arc, vec};
use rand::{distr::{Distribution, weighted::WeightedIndex}, rngs::StdRng};

use crate::{canned_message::default_canned_message, pathfinder4::{DestinationSide, Distance, DistanceUnit, Guidance, Intersection, KeepSide, LaneGuidance, SignName, StopSign, TrafficLight, Turn, TurnSharpness, TurnSide}, scenarios::Scenario, voices::{SpeechText::{self, Seq, StaticText}, StaticVoice, Voice}};

const SCENARIO: &str = "六花とつむぎのスタンプラリー";

fn choose(weights: &[u32], rng: &mut StdRng) -> usize {
    let d = WeightedIndex::new(weights).unwrap();
    d.sample(rng)
}

#[derive(Debug, Clone)]
pub struct 六花とつむぎのスタンプラリーScenario {
    pub 六花: Arc<Voice>,
    pub ぴた声六花: Arc<Voice>,
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

    fn render_canned_message(&self, c: crate::canned_message::Canned) -> SpeechText {
        SpeechText::DynamicText(default_canned_message(c), self.つむぎ.clone())
    }

    fn static_text_catalog(&self) -> Vec<SpeechText> {
        let 六花 = |s: &str| StaticText(s.to_string(), self.六花.clone());
        let ぴた声六花 = |s: &str| StaticText(s.to_string(), self.ぴた声六花.clone());
        let 六花とつむぎ = |s: &str| StaticText(s.to_string(), self.六花とつむぎ.clone());
        vec![
            // ぴた声六花 — StraightStep
            ぴた声六花("KRTN0827_まっすぐ進んで下さい.wav"),
            ぴた声六花("KRTN0828_この先まっすぐです.wav"),
            ぴた声六花("KRTN0829_直進してください.wav"),
            ぴた声六花("KRTN0934_まっすぐ進んで.wav"),
            // ぴた声六花 — TurnStep
            ぴた声六花("KRTN0839_左に曲がります.wav"),
            ぴた声六花("KRTN0940_左に曲がって.wav"),
            ぴた声六花("KRTN0832_次、左にまがります.wav"),
            ぴた声六花("KRTN0838_右に曲がります.wav"),
            ぴた声六花("KRTN0939_右に曲がって.wav"),
            ぴた声六花("KRTN0831_次、右にまがります.wav"),

            // 六花とつむぎ — CombineMergedGuidanceEvents の「続いて」
            六花とつむぎ("続いて、0"),
            六花とつむぎ("続いて、1"),
            六花とつむぎ("続いて、2"),
            六花とつむぎ("続いて、3"),

            // 六花 — StraightStep レーン（postfix "を"）
            六花("左車線を"),
            六花("左側2車線を"),
            六花("右車線を"),
            六花("右側2車線を"),
            六花("真ん中の車線を"),
            六花("右から2番目の車線を"),
            六花("任意の車線を"),

            // 六花 — レーン（postfix "を使用して"、Turn/Ramp/Keep/Merge 系）
            六花("左車線を使用して"),
            六花("左側2車線を使用して"),
            六花("右車線を使用して"),
            六花("右側2車線を使用して"),
            六花("真ん中の車線を使用して"),
            六花("右から2番目の車線を使用して"),
            六花("任意の車線を使用して"),

            // 六花 — 交差点（contains_dynamic=false のとき信号/標識のみ）
            六花("信号で"),
            六花("ずっと先の信号で"),
            六花("一時停止の標識で"),
            六花("ずっと先の一時停止の標識で"),

            // 六花 — TurnStep ターン方向（postfix "です"）
            六花("左方向です"),
            六花("右方向です"),
            六花("斜め左方向です"),
            六花("斜め右方向です"),
            六花("左手前方向です"),
            六花("右手前方向です"),

            // 六花 — OnRampStep ターン方向（postfix "に進み"）
            六花("左方向に進み"),
            六花("右方向に進み"),
            六花("斜め左方向に進み"),
            六花("斜め右方向に進み"),
            六花("左手前方向に進み"),
            六花("右手前方向に進み"),

            // 六花 — 固定フレーズ
            六花("Uターンします。"),
            六花("ランプに進みます。"),
            六花("出口を出ます。"),
            六花("分岐を左方向です。"),
            六花("分岐を右方向です。"),
            六花("合流します。"),
            六花("目的地は左側です。"),
            六花("目的地は右側です。"),
            六花("まもなく目的地です。"),
            六花("目的地に到着しました"),

            // 六花 — CombineMergedGuidanceEvents（六花同士）の「続いて」
            六花("続いて、0"),
            六花("続いて、1"),

            // 六花 — ContinueForDistance（固定距離のみ）
            六花("およそ100メートル道なりです。"),
            六花("およそ150メートル道なりです。"),
            六花("およそ200メートル道なりです。"),
            六花("およそ300メートル道なりです。"),
            六花("およそ400メートル道なりです。"),
            六花("およそ500メートル道なりです。"),
            六花("およそ600メートル道なりです。"),
            六花("およそ700メートル道なりです。"),
            六花("およそ800メートル道なりです。"),
            六花("およそ900メートル道なりです。"),
            六花("およそ1キロ道なりです。"),
            六花("およそ1.5キロ道なりです。"),
            六花("およそ2キロ道なりです。"),
            六花("およそ3キロ道なりです。"),

            // 六花 — PrepareDistanceMessage の距離プレフィックス
            六花("およそ100メートル先、"),
            六花("およそ150メートル先、"),
            六花("およそ200メートル先、"),
            六花("およそ300メートル先、"),
            六花("およそ400メートル先、"),
            六花("およそ500メートル先、"),
            六花("およそ600メートル先、"),
            六花("およそ700メートル先、"),
            六花("およそ800メートル先、"),
            六花("およそ900メートル先、"),
            六花("およそ1キロ先、"),
            六花("およそ1.5キロ先、"),
            六花("およそ2キロ先、"),
            六花("およそ3キロ先、"),
        ]
    }
}

fn contains_dynamic(g: &Guidance) -> bool {
    fn intersection(i: &Intersection) -> bool {
        i.name.is_some()
    }
    fn sign_name(s: &SignName) -> bool {
        s.direct.is_some() || s.indirect.is_some()
    }
    fn distance(d: &Distance) -> bool {
        match (d.value, d.unit) {
            (100.0, DistanceUnit::Meter) => false,
            (150.0, DistanceUnit::Meter) => false,
            (200.0, DistanceUnit::Meter) => false,
            (300.0, DistanceUnit::Meter) => false,
            (400.0, DistanceUnit::Meter) => false,
            (500.0, DistanceUnit::Meter) => false,
            (600.0, DistanceUnit::Meter) => false,
            (700.0, DistanceUnit::Meter) => false,
            (800.0, DistanceUnit::Meter) => false,
            (900.0, DistanceUnit::Meter) => false,
            (1.0, DistanceUnit::Kilometer) => false,
            (1.5, DistanceUnit::Kilometer) => false,
            (2.0, DistanceUnit::Kilometer) => false,
            (3.0, DistanceUnit::Kilometer) => false,
            _ => true
        }
    }
    match g {
        Guidance::StraightStep(_) => false,
        Guidance::TurnStep(_, _, i) => intersection(i),
        Guidance::UTurnStep(i) => intersection(i),
        Guidance::OnRampStep(_, _, i, s) => intersection(i) || sign_name(s),
        Guidance::OffRampStep(_, e, s) => e.is_some() || sign_name(s),
        Guidance::KeepOrForkStep(_, _) => false,
        Guidance::MergeStep(_) => false,
        Guidance::InterchangeStep(_, _, _) => true,
        Guidance::DestinationStepPrepare(_) => false,
        Guidance::DestinationStepAct => false,
        Guidance::ContinueForDistance(d) => distance(d),
        Guidance::PrepareDistanceMessage(d, g) => distance(d) || contains_dynamic(g),
        Guidance::CombineMergedGuidanceEvents(first, second) => contains_dynamic(first) || contains_dynamic(second)
    }
}

fn render_lane(lane: &LaneGuidance, postfix: &str) -> String {
    let lane = match lane {
        LaneGuidance::LeftLane => "左車線",
        LaneGuidance::Left2Lanes => "左側2車線",
        LaneGuidance::RightLane => "右車線",
        LaneGuidance::Right2Lanes => "右側2車線",
        LaneGuidance::MiddleLane => "真ん中の車線",
        LaneGuidance::SecondFromRight => "右から2番目の車線",
        LaneGuidance::AnyLane => "任意の車線",
    };
    format!("{}{}", lane, postfix)
}

fn render_opt_lane(opt_lane: &Option<LaneGuidance>, postfix: &str) -> String {
    opt_lane
        .as_ref()
        .map(|lane|render_lane(lane, postfix))
        .unwrap_or_default()
}

fn render_intersection(intersection: &Intersection, postfix: &str) -> String {
    let s = if let Some(stop) = &intersection.stop_sign {
        match stop {
            StopSign { index: -1 } => "一時停止の標識",
            StopSign { index: _ } => "ずっと先の一時停止の標識",
        }.to_string()
    } else if let Some(tlight) = &intersection.traffic_light {
        match tlight {
            TrafficLight { index: -1 } => "信号",
            TrafficLight { index: _ } => "ずっと先の信号"
        }.to_string()
    } else if let Some(name) = &intersection.name {
        name.replace("（交差点）", "交差点")
    } else {
        return Default::default()
    };
    format!("{}{}", s, postfix)
}

fn render_turn(turn: &Turn, postfix: &str) -> String {
    let s = match (turn.sharpness, turn.side) {
            (TurnSharpness::Normal, TurnSide::Left) => "左方向",
            (TurnSharpness::Normal, TurnSide::Right) => "右方向",
            (TurnSharpness::Slight, TurnSide::Left) => "斜め左方向",
            (TurnSharpness::Slight, TurnSide::Right) => "斜め右方向",
            (TurnSharpness::Sharp, TurnSide::Left) => "左手前方向",
            (TurnSharpness::Sharp, TurnSide::Right) => "右手前方向",
    };
    format!("{}{}", s, postfix)
}

fn render_opt_turn(opt_turn: &Option<Turn>, postfix: &str) -> String {
    opt_turn.as_ref().map(|turn| render_turn(turn, postfix)).unwrap_or_default()
}

fn render_sign(sign: &SignName, postfix: &str) -> String {
    match (sign.indirect.as_ref(), sign.direct.as_ref()) {
        (Some(indirect), Some(direct)) => format!("{}方面{}{}", indirect, direct, postfix),
        (Some(indirect), None) => format!("{}方面{}", indirect, postfix),
        (None, Some(direct)) => format!("{}{}", direct, postfix),
        (None, None) => Default::default()
    }
}

fn render_distance(distance: &Distance) -> String {
    let u = match distance.unit {
        DistanceUnit::Meter => "メートル",
        DistanceUnit::Kilometer => "キロ",
        DistanceUnit::Mile => "マイル",
    };
    format!("{}{}", distance.value, u)
}

impl 六花とつむぎのスタンプラリーScenario {
    pub fn new(つむぎ: Arc<Voice>) -> Self {
        Self {
            六花: Arc::new(Voice::Static(StaticVoice::new(SCENARIO, "六花"))),
            ぴた声六花: Arc::new(Voice::Static(StaticVoice::new(SCENARIO, "ぴた声六花"))),
            つむぎ,
            六花とつむぎ: Arc::new(Voice::Static(StaticVoice::new(SCENARIO, "六花とつむぎ"))),
        }
    }


    fn render_guidance(&self, g: &Guidance, rng: &mut StdRng) -> SpeechText {
        // まず誰がしゃべるか決める
        match g {
            Guidance::CombineMergedGuidanceEvents(first, second) => {
                // A。続いて、Bの場合は前半後半でわけてもいい
                let aつむぎ = contains_dynamic(first) || choose(&[2, 1], rng) == 1;
                let bつむぎ = contains_dynamic(second) || choose(&[2, 1], rng) == 1;
                match (aつむぎ, bつむぎ) {
                    (true, true) => {
                        // 全部つむぎ
                        self.render_つむぎ_guidance(g, rng)
                    }
                    // 途中で交代する場合の「続いて」は2人で読む
                    (true, false) => {
                        let then = match choose(&[1, 1, 1, 1], rng) {
                            0 => "続いて、0",
                            1 => "続いて、1",
                            2 => "続いて、2",
                            _ => "続いて、3",
                        };
                        SpeechText::Seq(vec![
                            self.render_つむぎ_guidance(first, rng),
                            StaticText(then.to_string(), self.六花とつむぎ.clone()),
                            self.render_六花_guidance(second, rng)
                        ])
                    }
                    (false, true) => {
                        let then = match choose(&[1, 1, 1, 1], rng) {
                            0 => "続いて、0",
                            1 => "続いて、1",
                            2 => "続いて、2",
                            _ => "続いて、3",
                        };
                        SpeechText::Seq(vec![
                            self.render_六花_guidance(first, rng),
                            StaticText(then.to_string(), self.六花とつむぎ.clone()),
                            self.render_つむぎ_guidance(second, rng)
                            
                        ])
                    }
                    (false, false) => {
                        let then = match choose(&[1, 1], rng) {
                            0 => "続いて、0",
                            _ => "続いて、1"
                        };
                        SpeechText::Seq(vec![
                            self.render_六花_guidance(first, rng),
                            StaticText(then.to_string(), self.六花.clone()),
                            self.render_六花_guidance(second, rng),
                        ])
                    }
                }
            }
            _ => {
                if contains_dynamic(g) {
                    self.render_つむぎ_guidance(g, rng)
                } else {
                    match choose(&[2, 1], rng) {
                        0 => self.render_六花_guidance(g, rng),
                        _ => self.render_つむぎ_guidance(g, rng)
                    }
                }
            }
        }
    }

    fn render_つむぎ_guidance(&self, g: &Guidance, rng: &mut StdRng) -> SpeechText {
        match g {
            Guidance::StraightStep(opt_lane) => {
                let s = format!("{}直進します。", render_opt_lane(opt_lane, "を"));
                StaticText(s, self.つむぎ.clone())
            }
            Guidance::TurnStep(turn, opt_lane, intersection) => {
                let i = render_intersection(intersection, "で");
                let lane = render_opt_lane(opt_lane, "を使用して");
                let turn = render_turn(turn, "です");
                let s = format!("{}{}{}。", lane, i, turn);
                StaticText(s, self.つむぎ.clone())
            }
            Guidance::UTurnStep(intersection) => {
                let i_name = intersection.name.as_deref()
                    .map(|name| format!("{}で、", name))
                    .unwrap_or("".to_string());
                let s = format!("{}Uターンします。", i_name);
                StaticText(s, self.つむぎ.clone())
            }
            Guidance::OnRampStep(opt_turn, opt_lane, intersection, sign) => {
                let i = render_intersection(intersection, "で");
                let lane = render_opt_lane(opt_lane, "を使用して");
                let turn = render_opt_turn(opt_turn, "に進み");
                let sign = render_sign(sign, "の");
                let s = format!("{}{}{}{}ランプに進みます。", lane, i, turn, sign);
                StaticText(s, self.つむぎ.clone())
            }
            Guidance::OffRampStep(opt_lane, e_name, sign) => {
                let lane = render_opt_lane(opt_lane, "を使用して");
                let e_name = e_name.as_ref().map(|name| format!("{}の", name.name)).unwrap_or_default();
                let sign = render_sign(sign, "");
                let s = format!("{}{}{}出口を出ます。", lane, sign, e_name).replace("インターチェンジの出口", "出口");
                StaticText(s, self.つむぎ.clone())
            }
            Guidance::KeepOrForkStep(keep, opt_lane) => {
                let lane = render_opt_lane(opt_lane, "を使用して");
                let keep = match keep {
                    KeepSide::Left => "左側を",
                    KeepSide::Right => "右側を",
                };
                let s = format!("{}{}進みます。", lane, keep);
                StaticText(s, self.つむぎ.clone())
            }
            Guidance::MergeStep(opt_lane) => {
                let lane = render_opt_lane(opt_lane, "を進み");
                let s = format!("{}合流します。", lane);
                StaticText(s, self.つむぎ.clone())
            }
            Guidance::DestinationStepPrepare(opt_side) => {
                let s = match opt_side {
                    Some(DestinationSide::Left) => "目的地は左側です。",
                    Some(DestinationSide::Right) => "目的地は右側です。",
                    _ => "まもなく目的地です。"
                }.to_string();
                StaticText(s, self.つむぎ.clone())
            }
            Guidance::InterchangeStep(opt_lane, i_name, sign) => {
                let lane = render_opt_lane(opt_lane, "を使用して");
                let i_name = i_name.name.clone();
                let sign = render_sign(sign, "");
                let s = format!("{}{}で{}へ進みます。", lane, i_name, sign);
                StaticText(s, self.つむぎ.clone())
            }
            Guidance::DestinationStepAct => {
                let s = "目的地に到着しました".to_string();
                StaticText(s, self.つむぎ.clone())
            }
            Guidance::ContinueForDistance(dist) => {
                let d = render_distance(dist);
                let s = format!("およそ{}道なりです。", d);
                StaticText(s, self.つむぎ.clone())
            }
            Guidance::PrepareDistanceMessage(dist, guidance) => {
                let d = render_distance(dist);
                let next = self.render_つむぎ_guidance(guidance, rng);
                let s = format!("およそ{}先、", d);
                SpeechText::Seq(vec![
                    StaticText(s, self.つむぎ.clone()), next
                ])
            }
            Guidance::CombineMergedGuidanceEvents(first, second ) => {
                let f = self.render_つむぎ_guidance(first, rng);
                let s = self.render_つむぎ_guidance(second, rng);
                SpeechText::Seq(vec![
                    f,
                    StaticText("続いて、".to_string(), self.つむぎ.clone()),
                    s
                ])
            }
        }
    }

    fn render_六花_guidance(&self, g: &Guidance, rng: &mut StdRng) -> SpeechText {
        let ぴた声六花 = |s: &str| StaticText(s.to_string(), self.ぴた声六花.clone());
        match g {
            Guidance::StraightStep(opt_lane) => {
                let v = match choose(&[1, 1, 1, 1], rng) {
                    0 => ぴた声六花("KRTN0827_まっすぐ進んで下さい.wav"),
                    1 => ぴた声六花("KRTN0828_この先まっすぐです.wav"),
                    2 => ぴた声六花("KRTN0829_直進してください.wav"),
                    _ => ぴた声六花("KRTN0934_まっすぐ進んで.wav"),
                };
                if let Some(lane) = opt_lane {
                    // {lane}を直進します。
                    let lane = render_lane(lane, "を");
                    Seq(vec![StaticText(lane, self.六花.clone()), v])
                } else {
                    v                    
                }
            }
            Guidance::TurnStep(turn, opt_lane, intersection) => {
                let mut seq = Vec::new();
                let i = render_intersection(intersection, "で");
                if !i.is_empty() {
                    seq.push(StaticText(i, self.六花.clone()))
                }
                let lane = render_opt_lane(opt_lane, "を使用して");
                if !lane.is_empty() {
                    seq.push(StaticText(lane, self.六花.clone()))
                }
                let turn = render_turn(turn, "です");
                let turn = match turn.as_str() {
                    "左方向です" => match choose(&[1, 1, 1], rng) {
                        0 => ぴた声六花("KRTN0839_左に曲がります.wav"),
                        1 => ぴた声六花("KRTN0940_左に曲がって.wav"),
                        2 if seq.is_empty() => ぴた声六花("KRTN0832_次、左にまがります.wav"),
                        _ => StaticText(turn, self.六花.clone()),
                    }
                    "右方向です" => match choose(&[1, 1, 1], rng) {
                        0 => ぴた声六花("KRTN0838_右に曲がります.wav"),
                        1 => ぴた声六花("KRTN0939_右に曲がって.wav"),
                        2 if seq.is_empty() => ぴた声六花("KRTN0831_次、右にまがります.wav"),
                        _ => StaticText(turn, self.六花.clone()),
                    }
                    _ => StaticText(turn, self.六花.clone())
                };
                seq.push(turn);
                Seq(seq)
            }
            Guidance::UTurnStep(intersection) => {
                let mut seq = Vec::new();
                let i = render_intersection(intersection, "で");
                if !i.is_empty() {
                    seq.push(StaticText(i, self.六花.clone()))
                }
                seq.push(StaticText("Uターンします。".to_string(), self.六花.clone()));
                Seq(seq)
            }
            Guidance::OnRampStep(opt_turn, opt_lane, intersection, _) => {
                let mut seq = Vec::new();
                let i = render_intersection(intersection, "で");
                if !i.is_empty() {
                    seq.push(StaticText(i, self.六花.clone()))
                }
                let lane = render_opt_lane(opt_lane, "を使用して");
                if !lane.is_empty() {
                    seq.push(StaticText(lane, self.六花.clone()))
                }
                let turn = render_opt_turn(opt_turn, "に進み");
                if !turn.is_empty() {
                    seq.push(StaticText(turn, self.六花.clone()))
                }
                let ramp = StaticText("ランプに進みます。".to_string(), self.六花.clone());
                seq.push(ramp);
                Seq(seq)
            }
            Guidance::OffRampStep(opt_lane, _, _) => {
                let mut seq = Vec::new();
                let lane = render_opt_lane(opt_lane, "を使用して");
                if !lane.is_empty() {
                    seq.push(StaticText(lane, self.六花.clone()))
                }
                let ramp = StaticText("出口を出ます。".to_string(), self.六花.clone());
                seq.push(ramp);
                Seq(seq)
            }
            Guidance::KeepOrForkStep(keep, opt_lane) => {
                let mut seq = Vec::new();
                let lane = render_opt_lane(opt_lane, "を使用して");
                if !lane.is_empty() {
                    seq.push(StaticText(lane, self.六花.clone()))
                }
                let keep = match keep {
                    KeepSide::Left => "分岐を左方向です。",
                    KeepSide::Right => "分岐を右方向です。",
                };
                seq.push(StaticText(keep.to_string(), self.六花.clone()));
                Seq(seq)
            }
            Guidance::MergeStep(opt_lane) => {
                let mut seq = Vec::new();
                let lane = render_opt_lane(opt_lane, "を使用して");
                if !lane.is_empty() {
                    seq.push(StaticText(lane, self.六花.clone()))
                }
                let merge = StaticText("合流します。".to_string(), self.六花.clone());
                seq.push(merge);
                Seq(seq)
            }
            Guidance::DestinationStepPrepare(opt_side) => {
                let s = match opt_side {
                    Some(DestinationSide::Left) => "目的地は左側です。",
                    Some(DestinationSide::Right) => "目的地は右側です。",
                    _ => "まもなく目的地です。"
                }.to_string();
                StaticText(s, self.六花.clone())
            }
            Guidance::InterchangeStep(_, _, _) => {
                unreachable!()
            }
            Guidance::DestinationStepAct => {
                let s = "目的地に到着しました".to_string();
                StaticText(s, self.六花.clone())
            }
            Guidance::ContinueForDistance(dist) => {
                let d = render_distance(dist);
                let s = format!("およそ{}道なりです。", d);
                StaticText(s, self.六花.clone())
            }
            Guidance::PrepareDistanceMessage(dist, guidance) => {
                let d = render_distance(dist);
                let next = self.render_六花_guidance(guidance, rng);
                let s = format!("およそ{}先、", d);
                SpeechText::Seq(vec![
                    StaticText(s, self.六花.clone()), next
                ])
            }
            Guidance::CombineMergedGuidanceEvents(_first, _second ) => {
                unreachable!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voices::{Speaker, SpeakerStyle};
    use crate::pathfinder4::Guidance;
    use crate::scenarios::Scenario;
    use crate::voices::SpeechText;
    use std::collections::HashSet;

    fn make_scenario() -> 六花とつむぎのスタンプラリーScenario {
        let つむぎ = Arc::new(Voice::VOICEVOX(
            Speaker {
                name: "つむぎ".to_string(),
                speaker_uuid: "dummy".to_string(),
                styles: vec![],
                version: "0".to_string(),
                supported_features: crate::voices::voicevox::SupportedFeatures {
                    permitted_synthesis_morphing: "ALL".to_string(),
                },
            },
            SpeakerStyle { id: 8, name: "ノーマル".to_string(), style_type: "talk".to_string() },
        ));
        六花とつむぎのスタンプラリーScenario::new(つむぎ)
    }

    fn collect_static_texts(text: &SpeechText) -> Vec<(String, String)> {
        match text {
            SpeechText::StaticText(t, voice) => match voice.as_ref() {
                Voice::Static(_) => vec![(t.clone(), voice.name())],
                _ => vec![],
            },
            SpeechText::DynamicText(_, _) => vec![],
            SpeechText::Seq(children) => children.iter()
                .flat_map(collect_static_texts)
                .collect(),
        }
    }

    #[test]
    fn catalog_covers_all_rendered_static_texts() {
        let scenario = make_scenario();

        let catalog: HashSet<(String, String)> = scenario
            .static_text_catalog()
            .iter()
            .flat_map(collect_static_texts)
            .collect();

        for guidance in Guidance::all_variants() {
            for seed in 0u64..100 {
                use rand::{SeedableRng, rngs::StdRng};
                let mut rng = StdRng::seed_from_u64(seed);
                let result = scenario.render_guidance(&guidance, &mut rng);
                for (text, voice_name) in collect_static_texts(&result) {
                    assert!(
                        catalog.contains(&(text.clone(), voice_name.clone())),
                        "カタログ未登録: {:?} [{}]",
                        text, voice_name
                    );
                }
            }
        }
    }
}
