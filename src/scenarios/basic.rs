use crate::pathfinder4::{Guidance, LaneGuidance, TurnSharpness, TurnSide};

pub fn format_guidance(g: &Guidance) -> String {
    match g {
        Guidance::StraightStep(opt_lane) => {
            if let Some(lane) = opt_lane {
                match lane {
                    LaneGuidance::LeftLane => "まっすぐ（左車線）に進みます".to_string(),
                    LaneGuidance::RightLane => "まっすぐ（右車線）に進みます".to_string(),
                    _ => "まっすぐ進みます".to_string(),
                }
            } else {
                "まっすぐ進みます".to_string()
            }
        }
        Guidance::TurnStep(sharpness, side, _lane, _iname, _tlight, _stop) => {
            let side_s = match side {
                TurnSide::Left => "左",
                TurnSide::Right => "右",
            };
            let sharp_s = match sharpness {
                TurnSharpness::Slight => "やや",
                TurnSharpness::Normal => "",
                TurnSharpness::Sharp => "鋭く",
            };
            format!("{}{}方向に曲がります", sharp_s, side_s)
        }
        Guidance::UTurnStep(_) => "Uターンします".to_string(),
        Guidance::KeepOrForkStep(_keep, _lane) => "進路を維持するか分岐します".to_string(),
        Guidance::MergeStep(_) => "合流します".to_string(),
        Guidance::DestinationStepPrepare(_) => "目的地に向けて準備します".to_string(),
        Guidance::DestinationStepAct => "目的地に到着しました".to_string(),
        Guidance::ContinueForDistance(dist, unit, _ovr) => {
            let unit_s = match unit {
                crate::pathfinder4::DistanceUnit::Meter => "m",
                crate::pathfinder4::DistanceUnit::Kilometer => "km",
                crate::pathfinder4::DistanceUnit::Mile => "mi",
            };
            format!("約 {}{} 進みます", dist, unit_s)
        }
        other => format!("{:?}", other),
    }
}
