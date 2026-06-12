use crate::pathfinder4::{DestinationSide, DistanceUnit, Guidance, Heading, Intersection, KeepSide, LaneGuidance, SignName, StopSign, TrafficLight, Turn, TurnSharpness, TurnSide};

fn render_distance_unit(unit: DistanceUnit) -> &'static str {
    match unit {
        DistanceUnit::Meter => "メートル",
        DistanceUnit::Kilometer => "キロ",
        DistanceUnit::Mile => "マイル",
    }
}

pub fn default_render_guidance(g: &Guidance) -> Result<String, String> {
    fn render_heading(heading: Heading) -> String {
        match heading {
            Heading::North => "北",
            Heading::NorthEast => "北東",
            Heading::East => "東",
            Heading::SouthEast => "南東",
            Heading::South => "南",
            Heading::SouthWest => "南西",
            Heading::West => "西",
            Heading::NorthWest => "北西",
        }.to_string()
    }

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

    fn render_opt_lane(opt_lane: &Option<LaneGuidance>, postfix: &str) -> String {
        opt_lane
            .map(render_lane)
            .map(|lane| format!("{}{}", lane, postfix))
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

    match g {
        Guidance::DepartStep(heading) => {
            let h = render_heading(*heading);
            Ok(format!("{}に進みます。", h))
        }
        Guidance::StraightStep(opt_lane) => {
            Ok(format!("{}直進します。", render_opt_lane(opt_lane, "を")))
        }
        Guidance::TurnStep(turn, opt_lane, intersection) => {
            let i = render_intersection(intersection, "で");
            let lane = render_opt_lane(opt_lane, "を使用して");
            let turn = render_turn(turn, "です");
            Ok(format!("{}{}{}。", lane, i, turn))
        }
        Guidance::UTurnStep(intersection) => {
            let i_name = intersection.name.as_deref()
                .map(|name| format!("{}で、", name))
                .unwrap_or("".to_string());
            Ok(format!("{}Uターンします。", i_name))
        }
        Guidance::OnRampStep(opt_turn, opt_lane, intersection, sign) => {
            let i = render_intersection(intersection, "で");
            let lane = render_opt_lane(opt_lane, "を使用して");
            let turn = render_opt_turn(opt_turn, "に進み");
            let sign = render_sign(sign, "の");
            Ok(format!("{}{}{}{}ランプに進みます。", lane, i, turn, sign))
        }
        Guidance::OffRampStep(opt_lane, e_name, sign) => {
            let lane = render_opt_lane(opt_lane, "を使用して");
            let e_name = e_name.as_ref().map(|name| format!("{}の", name.name)).unwrap_or_default();
            let sign = render_sign(sign, "");
            Ok(format!("{}{}{}出口を出ます。", lane, sign, e_name).replace("インターチェンジの出口", "出口"))
        }
        Guidance::KeepOrForkStep(keep, opt_lane) => {
            let lane = render_opt_lane(opt_lane, "を使用して");
            let keep = match keep {
                KeepSide::Left => "左側を",
                KeepSide::Right => "右側を",
            };
            Ok(format!("{}{}進みます。", lane, keep))
        }
        Guidance::MergeStep(opt_lane) => {
            let lane = render_opt_lane(opt_lane, "を進み");
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
            let lane = render_opt_lane(opt_lane, "を使用して");
            let i_name = i_name.name.clone();
            let sign = render_sign(sign, "");
            Ok(format!("{}{}で{}へ進みます。", lane, i_name, sign))
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
            Ok(format!("{}続いて、{}", f, s))
        }
    }
}
