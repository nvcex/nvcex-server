use crate::pathfinder4::{DestinationSide, DistanceUnit, Guidance, KeepSide, LaneGuidance, StopSign, TrafficLight, Turn, TurnSharpness, TurnSide};

fn render_distance_unit(unit: DistanceUnit) -> &'static str {
    match unit {
        DistanceUnit::Meter => "メートル",
        DistanceUnit::Kilometer => "キロ",
        DistanceUnit::Mile => "マイル",
    }
}

pub fn default_render_guidance(g: &Guidance) -> Result<String, String> {
    fn render_lane(lane: LaneGuidance) -> String {
        match lane {
            LaneGuidance::LeftLane => "左車線",
            LaneGuidance::Left2Lanes => "左側2車線",
            LaneGuidance::RightLane => "右車線",
            LaneGuidance::Right2Lanes => "右側2車線",
            LaneGuidance::MiddleLane => "真ん中の車線",
            LaneGuidance::SecondFromRight => "右から2番目の車線",
            LaneGuidance::AnyLane => "任意の車線",
        }.to_string()        
    }

    match g {
        Guidance::StraightStep(opt_lane) => {
            let lane = opt_lane
                .map(render_lane)
                .map(|lane| format!("{}を", lane))
                .unwrap_or("".to_string());
            Ok(format!("{}直進します。", lane))
        }
        Guidance::TurnStep(Turn { sharpness, side }, opt_lane, intersection) => {
            let stop = match intersection.stop_sign.as_ref() {
                Some(StopSign { index: -1 }) => "一時停止の標識で、",
                Some(StopSign { index: _ }) => "ずっと先の一時停止の標識で、",
                _ => ""
            }.to_string();
            let tlight = match intersection.traffic_light.as_ref() {
                Some(TrafficLight { index: -1 }) => "信号で、",
                Some(TrafficLight { index: _ }) => "ずっと先の信号で、",
                _ => ""
            }.to_string();
            let i_name = intersection.name.as_deref()
                .map(|name| format!("{}を、", name))
                .unwrap_or("".to_string());
            let lane = opt_lane
                .map(render_lane)
                .map(|lane| format!("{}を使用して、", lane))
                .unwrap_or("".to_string());
            let side = match (sharpness, side) {
                (TurnSharpness::Normal, TurnSide::Left) => "左折します",
                (TurnSharpness::Normal, TurnSide::Right) => "右折します",
                (TurnSharpness::Slight, TurnSide::Left) => "斜め左方向です",
                (TurnSharpness::Slight, TurnSide::Right) => "斜め右方向です",
                (TurnSharpness::Sharp, TurnSide::Left) => "左手前方向です",
                (TurnSharpness::Sharp, TurnSide::Right) => "右手前方向です",
            };
            Ok(format!("{}{}{}{}{}。", stop, tlight, i_name, lane, side))
        }
        Guidance::UTurnStep(intersection) => {
            let i_name = intersection.name.as_deref()
                .map(|name| format!("{}で、", name))
                .unwrap_or("".to_string());
            Ok(format!("{}Uターンします。", i_name))
        }
        Guidance::OnRampStep(opt_turn, opt_lane, intersection, sign) => {
            let tlight = match intersection.traffic_light.as_ref() {
                Some(TrafficLight { index: -1 }) => "信号で、",
                Some(TrafficLight { index: _ }) => "ずっと先の信号で、",
                _ => ""
            }.to_string();
            let lane = opt_lane
                .map(render_lane)
                .map(|lane| format!("{}を使用して、", lane))
                .unwrap_or("".to_string());
            let side = opt_turn.map(|Turn { sharpness, side }| match (sharpness, side) {
                (TurnSharpness::Normal, TurnSide::Left) => "左折し",
                (TurnSharpness::Normal, TurnSide::Right) => "右折し",
                (TurnSharpness::Slight, TurnSide::Left) => "斜め左方向へ進み",
                (TurnSharpness::Slight, TurnSide::Right) => "斜め右方向へ進み",
                (TurnSharpness::Sharp, TurnSide::Left) => "左手前方向へ進み",
                (TurnSharpness::Sharp, TurnSide::Right) => "右手前方向へ進み",
            }).unwrap_or("").to_string();
            let iname = intersection.name.as_deref()
                .map(|name| format!("{}を", name))
                .unwrap_or("".to_string());
            let sd_name = sign.direct.as_deref().unwrap_or("");
            let si_name = sign.indirect.as_deref().unwrap_or("");
            Ok(format!("{}{}{}{}{}{}入ります。", tlight, lane, side, iname, sd_name, si_name))
        }
        Guidance::OffRampStep(opt_lane, e_name, sign) => {
            let lane = opt_lane
                .map(render_lane)
                .map(|lane| format!("{}を使用して、", lane))
                .unwrap_or("".to_string());
            let e_name = e_name.as_ref().map(|name| format!("{}を", name.name)).unwrap_or("".to_string());
            let sd_name = sign.direct.as_deref().unwrap_or("");
            let si_name = sign.indirect.as_deref().unwrap_or("");
            Ok(format!("{}{}{}{}を出ます。", lane, e_name, sd_name, si_name))
        }
        Guidance::KeepOrForkStep(keep, opt_lane) => {
            let keep = match keep {
                KeepSide::Left => "左側を",
                KeepSide::Right => "右側を",
            };
            let lane = opt_lane
                .map(render_lane)
                .map(|lane| format!("{}を使用して、", lane))
                .unwrap_or("".to_string());
            Ok(format!("{}{}進みます。", lane, keep))
        }
        Guidance::MergeStep(opt_lane) => {
            let lane = opt_lane
                .map(render_lane)
                .map(|lane| format!("{}を使用して", lane))
                .unwrap_or("".to_string());
            Ok(format!("{}合流します。", lane))
        }
        Guidance::DestinationStepPrepare(opt_side) => {
            let s = match opt_side {
                Some(DestinationSide::Left) => "目的地は左側です。",
                Some(DestinationSide::Right) => "目的地は右側です。",
                _ => "まもなく目的地です。"
            }.to_string();
            Ok(s)
        }
        Guidance::InterchangeStep(opt_lane, i_name, sign) => {
            let lane = opt_lane
                .map(render_lane)
                .map(|lane| format!("{}を使用して", lane))
                .unwrap_or("".to_string());
            let i_name = i_name.name.clone();
            let sd_name = sign.direct.as_deref().unwrap_or("");
            let si_name = sign.indirect.as_deref().unwrap_or("");
            Ok(format!("{}{}を{}{}出ます。", lane, i_name, sd_name, si_name))
        }
        Guidance::DestinationStepAct => {
            Ok("目的地に到着しました".to_string())
        }
        Guidance::ContinueForDistance(dist) => {
            let unit_s = render_distance_unit(dist.unit);
            Ok(format!("およそ{}{}道なりです。", dist.value, unit_s))
        }
        Guidance::PrepareDistanceMessage(dist, guidance) => {
            let unit_s = render_distance_unit(dist.unit);
            let s = default_render_guidance(guidance)?;
            Ok(format!("およそ{}{}先、{}", dist.value, unit_s, s))
        }
        Guidance::CombineMergedGuidanceEvents(first, second ) => {
            let f = default_render_guidance(first)?;
            let s = default_render_guidance(second)?;
            Ok(format!("{}つづいて、{}", f, s))
        }
    }
}
