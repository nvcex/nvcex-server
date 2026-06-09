use protobuf_core::{AsRefExtProtobuf, Field, FieldValue};
use std::str;

#[derive(Debug)]
pub enum Guidance {
    StraightStep(Option<LaneGuidance>),
    TurnStep(Turn, Option<LaneGuidance>, Intersection),
    UTurnStep(Intersection),
    OnRampStep(Option<Turn>, Option<LaneGuidance>, Intersection, SignName),
    OffRampStep(Option<LaneGuidance>, Option<ExitName>, SignName),
    KeepOrForkStep(KeepSide, Option<LaneGuidance>),
    MergeStep(Option<LaneGuidance>),
    InterchangeStep(Option<LaneGuidance>, InterchangeName, SignName),
    DestinationStepPrepare(Option<DestinationSide>),
    DestinationStepAct,
    ContinueForDistance(Distance),
    PrepareDistanceMessage(Distance, Box<Guidance>),
    CombineMergedGuidanceEvents(Box<Guidance>, Box<Guidance>),    
}

impl Guidance {
    // テスト用に用いるガイダンスの例たち
    pub fn all_variants() -> Vec<Guidance> {
        use itertools::iproduct;

        let turns: Vec<_> = iproduct!(TurnSharpness::ALL, TurnSide::ALL)
            .map(|(&sharpness, &side)| Turn { sharpness, side })
            .collect();

        let lane_opts: Vec<Option<LaneGuidance>> = [None]
            .into_iter()
            .chain(LaneGuidance::ALL.iter().copied().map(Some))
            .collect();

        let intersections: Vec<Intersection> =
            [None, Some(TrafficLight { index: -1 }), Some(TrafficLight { index: 1 })]
                .into_iter()
                .map(|tl| Intersection { name: None, traffic_light: tl, stop_sign: None })
            .chain([Some(StopSign { index: -1 }), Some(StopSign { index: 1 })]
                .into_iter()
                .map(|ss| Intersection { name: None, stop_sign: ss, traffic_light: None }))
            .chain(std::iter::once(
                Intersection { name: Some("交差点".to_string()), traffic_light: None, stop_sign: None }
            ))
            .collect();

        let turn_steps: Vec<_> = iproduct!(turns.iter(), lane_opts.iter(), intersections.iter())
            .map(|(turn, lane, i)| Guidance::TurnStep(*turn, *lane, i.clone()))
            .collect();
        turn_steps
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DistanceUnit {
    Meter,
    Kilometer,
    Mile,
}

#[derive(Debug, Clone)]
pub struct Distance {
    pub value: f64,
    pub unit: DistanceUnit,
    #[allow(dead_code)]
    pub distance_override: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum TurnSharpness {
    Slight,
    Normal,
    Sharp,
}
impl TurnSharpness {
    pub const ALL: &'static [TurnSharpness] = &[TurnSharpness::Slight, TurnSharpness::Normal, TurnSharpness::Sharp];
}

#[derive(Debug, Clone, Copy)]
pub enum TurnSide {
    Left,
    Right,
}
impl TurnSide {
    pub const ALL: &'static [TurnSide] = &[TurnSide::Left, TurnSide::Right];
}

#[derive(Debug, Clone, Copy)]
pub struct Turn {
    pub sharpness: TurnSharpness,
    pub side: TurnSide,
}

#[derive(Debug, Clone, Copy)]
pub enum KeepSide {
    Left,
    Right,
}
impl KeepSide {
    pub const ALL: &'static [KeepSide] = &[KeepSide::Left, KeepSide::Right];
}

#[derive(Debug, Clone, Copy)]
pub enum DestinationSide {
    Left,
    Right,
}
impl DestinationSide {
    pub const ALL: &'static [DestinationSide] = &[DestinationSide::Left, DestinationSide::Right];
}

#[derive(Debug, Clone, Copy)]
pub enum LaneGuidance {
    LeftLane,
    Left2Lanes,
    MiddleLane,
    RightLane,
    Right2Lanes,
    SecondFromRight,
    AnyLane,
}
impl LaneGuidance {
    pub const ALL: &'static [LaneGuidance] = &[
        LaneGuidance::LeftLane, LaneGuidance::Left2Lanes, LaneGuidance::MiddleLane,
        LaneGuidance::RightLane, LaneGuidance::Right2Lanes, LaneGuidance::SecondFromRight,
        LaneGuidance::AnyLane,
    ];
}

#[derive(Debug, Clone)]
pub struct TrafficLight {
    pub index: i64
}

#[derive(Debug, Clone)]
pub struct StopSign {
    pub index: i64
}

#[derive(Debug, Clone)]
pub struct Intersection {
    pub name: Option<String>,
    pub traffic_light: Option<TrafficLight>,
    pub stop_sign: Option<StopSign>,
}

#[derive(Debug)]
pub struct InterchangeName {
    pub name: String
}

#[derive(Debug)]
pub struct ExitName {
    pub name: String
}

#[derive(Debug)]
pub struct SignName {
    pub direct: Option<String>,
    pub indirect: Option<String>,
}

#[derive(Debug)]
struct Exits {
    names: Vec<NLGData>
}

#[derive(Debug)]
struct Routes {
    names: Vec<NLGData>
}

#[derive(Debug)]
struct NLGData {
    text: String
}

#[derive(Debug)]
enum Value {
    Distance(f64),
    DistanceUnit(DistanceUnit),
    DistanceOverride(String),
    TurnSharpness(TurnSharpness),
    TurnSide(TurnSide),
    KeepSide(KeepSide),
    DestinationSide(DestinationSide),
    LaneGuidance(LaneGuidance),
    TrafficLight(TrafficLight),
    StopSign(StopSign),
    ExitName(ExitName),
    Exits(Exits),
    Maneuver(Guidance),
    Key(PF4Enum),
    Args(Vec<Value>),
    FirstStep(Guidance),
    SecondStep(Guidance),
    SignDirectName(String),
    SignIndirectName(String),
    Routes(Routes),
    IntersectionName(String),
    InterchangeName(InterchangeName),
    NLGData(NLGData),
}

#[derive(Debug)]
enum PF4RawValue {
    // field 3: i64
    IntValue(i64),
    // field 4: Symbol
    SymbolValue(String),
    // field 6: f64
    FloatValue(f64),
    // field 11: PF4Enum
    EnumValue(PF4Enum),
    // field 14: PF4Command
    CommandValue(PF4Enum),
    // field 15: PF4NLGData
    NLGData(PF4NLGData),
    // field 16: PF4KeyValue
    KeyValueArray(Vec<Value>),
}

#[derive(Debug)]
struct PF4Enum {
    // field 1: string
    pub type_name: String,
    // field 2: string
    pub value: String,
}

#[derive(Debug)]
struct PF4NLGData {
    // field 1: string
    #[allow(dead_code)]
    pub id: Option<String>,
    // field 2: string
    pub text: String,
}

pub fn parse(bytes: &[u8]) -> Result<Guidance, String> {
    let mut body = None;
    let fields = parse_fields(bytes)?;
    for &Field { ref field_number, ref value } in &fields {
        match (field_number.as_u32(), value) {
            (2, &FieldValue::Len(nested_bytes)) if body.is_none()=> {
                body = Some(parse_guidance(nested_bytes)?);
            }
            (n, _) => return Err(format!("Unexpected field number {} in envelope", n))
        }
    }
    if let Some(body) = body {
        Ok(body)
    } else {
        Err("Missing required field 2 in StructuredMessage".to_string())
    }
}

fn parse_string(bytes: &[u8]) -> Result<String, String> {
    match str::from_utf8(bytes) {
        Ok(s) => Ok(s.to_string()),
        Err(e) => Err(format!("invalid utf-8 sequence: {}", e)),
    }
}

fn parse_guidance(bytes: &[u8]) -> Result<Guidance, String> {
    let mut args = None;
    let mut command = None;
    let fields = parse_fields(bytes)?;
    for &Field { ref field_number, ref value } in &fields {
            match (field_number.as_u32(), value) {
            (2, &FieldValue::Len(nested_bytes)) if args.is_none() => {
                args = Some(parse_values(nested_bytes)?);
            }
            (5, &FieldValue::Len(nested_bytes)) if command.is_none() => {
                command = Some(parse_command(nested_bytes)?);
            }
            (n, _) => return Err(format!("Unexpected field number {} in PF4Message", n))
        }
    }
    if let (Some(args), Some(command)) = (args, command) {
        map_guidance(command, args)
    } else {
        Err("Missing required fields in PF4Message".to_string())
    }
}

fn map_guidance(command: PF4Enum, args: Vec<Value>) -> Result<Guidance, String> {
    if command.type_name == "pathfinder4" {
        match command.value.as_str() {
            "pf_straightstep" => {
                let mut lane = None;
                for arg in args {
                    match arg {
                        Value::LaneGuidance(v) => lane = Some(v),
                        arg => return Err(format!("unknown args for pf_straightstep {:?}", arg))
                    }
                }
                Ok(Guidance::StraightStep(lane))
            }
            "pf_turnstep" => {
                let mut lane = None;
                let mut sharpness = None;
                let mut side = None;
                let mut intersection_name = None;
                let mut traffic_light = None;
                let mut stop_sign = None;
                for arg in args {
                    match arg {
                        Value::LaneGuidance(v) => lane = Some(v),
                        Value::TurnSharpness(v) => sharpness = Some(v),
                        Value::TurnSide(v) => side = Some(v),
                        Value::IntersectionName(v) => intersection_name = Some(v),
                        Value::TrafficLight(v) => traffic_light = Some(v),
                        Value::StopSign(v) => stop_sign = Some(v),
                        arg => return Err(format!("unknown args for pf_turnstep {:?}", arg))
                    }
                }
                if let (Some(sharpness), Some(side)) = (sharpness, side) {
                    Ok(Guidance::TurnStep(Turn { sharpness, side }, lane, Intersection { name: intersection_name, traffic_light, stop_sign }))
                } else {
                    Err("missing sharpness or side field in pf_turnstep".to_string())
                }
            }
            "pf_uturnstep" => {
                let mut intersection_name = None;
                for arg in args {
                    match arg {
                        Value::IntersectionName(v) => intersection_name = Some(v),
                        arg => return Err(format!("unknown args for pf_uturnstep {:?}", arg))
                    }
                }
                Ok(Guidance::UTurnStep(Intersection { name: intersection_name, traffic_light: None, stop_sign: None }))
            }
            "pf_onrampstep" => {
                let mut lane = None;
                let mut sharpness = None;
                let mut side = None;
                let mut intersection_name = None;
                let mut traffic_light = None;
                let mut sign_direct_name = None;
                let mut sign_indirect_name = None;
                for arg in args {
                    match arg {
                        Value::LaneGuidance(v) => lane = Some(v),
                        Value::TurnSharpness(v) => sharpness = Some(v),
                        Value::TurnSide(v) => side = Some(v),
                        Value::IntersectionName(v) => intersection_name = Some(v),
                        Value::TrafficLight(v) => traffic_light = Some(v),
                        Value::SignDirectName(v) => sign_direct_name = Some(v),
                        Value::SignIndirectName(v) => sign_indirect_name = Some(v),
                        arg => return Err(format!("unknown args for pf_onrampstep {:?}", arg))
                    }
                }
                let turn = match (sharpness, side) {
                    (Some(sharpness), Some(side)) => Some(Turn { sharpness, side }),
                    (None, None) => None,
                    _ => return Err("mismatched sharpness/side in pf_onrampstep".to_string()),
                };
                Ok(Guidance::OnRampStep(turn, lane, Intersection { name: intersection_name, traffic_light, stop_sign: None }, SignName { direct: sign_direct_name, indirect: sign_indirect_name }))
            }
            "pf_offrampstep" => {
                let mut lane = None;
                let mut exit = None;
                let mut sign_direct = None;
                let mut sign_indirect = None;
                for arg in args {
                    match arg {
                        Value::LaneGuidance(v) => lane = Some(v),
                        Value::ExitName(v) => exit = Some(v),
                        Value::SignDirectName(v) => sign_direct = Some(v),
                        Value::SignIndirectName(v) => sign_indirect = Some(v),
                        arg => return Err(format!("unknown args for pf_offrampstep {:?}", arg))
                    }
                }
                Ok(Guidance::OffRampStep(lane, exit, SignName { direct: sign_direct, indirect: sign_indirect }))
            }
            "pf_keeporforkstep" => {
                let mut lane = None;
                let mut side = None;
                for arg in args {
                    match arg {
                        Value::LaneGuidance(v) => lane = Some(v),
                        Value::KeepSide(v) => side = Some(v),
                        arg => return Err(format!("unknown args for pf_keeporforkstep {:?}", arg))
                    }
                }
                if let Some(side) = side {
                    Ok(Guidance::KeepOrForkStep(side, lane))
                } else {
                    Err("Missing keep field in pf_keeporforkstep".to_string())
                }
            }
            "pf_mergestep" => {
                let mut lane = None;
                for arg in args {
                    match arg {
                        Value::LaneGuidance(v) => lane = Some(v),
                        arg => return Err(format!("unknown args for pf_mergestep {:?}", arg))
                    }
                }
                Ok(Guidance::MergeStep(lane))
            }
            "pf_interchangestep" => {
                let mut lane = None;
                let mut name = None;
                let mut sign_direct_name = None;
                let mut sign_indirect_name = None;
                for arg in args {
                    match arg {
                        Value::LaneGuidance(v) => lane = Some(v),
                        Value::InterchangeName(v) => name = Some(v),
                        Value::SignDirectName(v) => sign_direct_name = Some(v),
                        Value::SignIndirectName(v) => sign_indirect_name = Some(v),
                        arg => return Err(format!("unknown args for pf_interchangestep {:?}", arg))
                    }
                }
                if let Some(name) = name {
                    Ok(Guidance::InterchangeStep(lane, name, SignName { direct: sign_direct_name, indirect: sign_indirect_name }))
                } else {
                    Err("Missing field in pf_interchangestep".to_string())
                }
            }
            "pf_destinationstep_prepare" => {
                let mut side = None;
                for arg in args {
                    match arg {
                        Value::DestinationSide(v) => side = Some(v),
                        arg => return Err(format!("unknown args for pf_destinationstep_prepare {:?}", arg))
                    }
                }
                Ok(Guidance::DestinationStepPrepare(side))
            }
            "pf_destinationstep_act" => match args.as_slice() {
                [] => Ok(Guidance::DestinationStepAct),
                values => Err(format!("unknown args for pf_destinationstep_act {:?}", values))
            }
            "continue_for_distance" => {
                let mut distance = None;
                let mut distance_unit = None;
                let mut distance_override = None;
                for arg in args {
                    match arg {
                        Value::Distance(v) => distance = Some(v),
                        Value::DistanceUnit(v) => distance_unit = Some(v),
                        Value::DistanceOverride(v) => distance_override = Some(v),
                        arg => return Err(format!("unknown args for continue_for_distance {:?}", arg))
                    }
                }
                if let (Some(value), Some(unit)) = (distance, distance_unit) {
                    Ok(Guidance::ContinueForDistance(Distance { value, unit, distance_override }))
                } else {
                    Err("Missing field in continue_for_distance".to_string())
                }
            }
            "prepare_distance_message" => {
                let mut distance = None;
                let mut distance_unit = None;
                let mut distance_override = None;
                let mut maneuver = None;
                for arg in args {
                    match arg {
                        Value::Distance(v) => distance = Some(v),
                        Value::DistanceUnit(v) => distance_unit = Some(v),
                        Value::DistanceOverride(v) => distance_override = Some(v),
                        Value::Maneuver(v) => maneuver = Some(v),
                        arg => return Err(format!("unknown args for prepare_distance_message {:?}", arg))
                    }
                }
                if let (Some(value), Some(unit), Some(maneuver)) = (distance, distance_unit, maneuver) {
                    Ok(Guidance::PrepareDistanceMessage(Distance { value, unit, distance_override }, Box::new(maneuver)))
                } else {
                    Err("Missing field in prepare_distance_message".to_string())
                }
            }
            "combine_merged_guidance_events" => {
                let mut first = None;
                let mut second = None;
                for arg in args {
                    match arg {
                        Value::FirstStep(v) => first = Some(v),
                        Value::SecondStep(v) => second = Some(v),
                        arg => return Err(format!("unknown args for combine_merged_guidance_events {:?}", arg))
                    }
                }
                if let (Some(first), Some(second)) = (first, second) {
                    Ok(Guidance::CombineMergedGuidanceEvents(Box::new(first), Box::new(second)))
                } else {
                    Err("Missing field in combine_merged_guidance_events".to_string())
                }
            }
            v => Err(format!("Unknown command value {}", v))
        }
    } else {
        Err(format!("Unknown command type {}", command.type_name))
    }
}

fn parse_values(bytes: &[u8]) -> Result<Vec<Value>, String> {
    let mut values = Vec::new();
    let fields = parse_fields(bytes)?;
    for &Field { ref field_number, ref value } in &fields {
        match (field_number.as_u32(), value) {
            (1, &FieldValue::Len(nested_bytes)) => {
                values.push(parse_value(nested_bytes)?);
            }
            (n, _) => return Err(format!("Unexpected field number {} in PF4MessageFields", n))
        }
    }
    Ok(values)
}

fn map_value(key: Option<String>, value: PF4RawValue) -> Result<Value, String> {
    if let Some(key) = key {
        match (key.as_str(), value) {
            ("distance", PF4RawValue::FloatValue(f)) => Ok(Value::Distance(f)),
            ("distance_unit", PF4RawValue::EnumValue(enum_value)) if enum_value.type_name == "nlp_generation.UnitType" => {
                match enum_value.value.as_str() {
                    "UNIT_METERS" => Ok(Value::DistanceUnit(DistanceUnit::Meter)),
                    "UNIT_KILOMETERS" => Ok(Value::DistanceUnit(DistanceUnit::Kilometer)),
                    "UNIT_MILES" => Ok(Value::DistanceUnit(DistanceUnit::Mile)),
                    _ => Err(format!("Unexpected value for field 'distance_unit': {}", enum_value.value))
                }
            }
            ("distance_override_type", PF4RawValue::SymbolValue(symbol)) => Ok(Value::DistanceOverride(symbol)),
            ("turn_sharpness", PF4RawValue::SymbolValue(symbol)) => {
                match symbol.as_str() {
                    "SLIGHT" => Ok(Value::TurnSharpness(TurnSharpness::Slight)),
                    "NORMAL" => Ok(Value::TurnSharpness(TurnSharpness::Normal)),
                    "SHARP" => Ok(Value::TurnSharpness(TurnSharpness::Sharp)),
                    _ => Err(format!("Unexpected value for field 'turn_sharpness': {}", symbol))
                }
            }
            ("turn_side", PF4RawValue::SymbolValue(symbol)) => {
                match symbol.as_str() {
                    "LEFT" => Ok(Value::TurnSide(TurnSide::Left)),
                    "RIGHT" => Ok(Value::TurnSide(TurnSide::Right)),
                    _ => Err(format!("Unexpected value for field 'turn_side': {}", symbol))
                }
            }
            ("keep_side", PF4RawValue::SymbolValue(symbol)) => {
                match symbol.as_str() {
                    "LEFT" => Ok(Value::KeepSide(KeepSide::Left)),
                    "RIGHT" => Ok(Value::KeepSide(KeepSide::Right)),
                    _ => Err(format!("Unexpected value for field 'keep_side': {}", symbol))
                }
            }
            ("destination_side", PF4RawValue::SymbolValue(symbol)) => {
                match symbol.as_str() {
                    "LEFT" => Ok(Value::DestinationSide(DestinationSide::Left)),
                    "RIGHT" => Ok(Value::DestinationSide(DestinationSide::Right)),
                    _ => Err(format!("Unexpected value for field 'destination_side': {}", symbol))
                }
            }
            ("lane_guidance_type", PF4RawValue::SymbolValue(symbol)) => {
                match symbol.as_str() {
                    "USE_LEFT_LANE" => Ok(Value::LaneGuidance(LaneGuidance::LeftLane)),
                    "USE_LEFT_2_LANES" => Ok(Value::LaneGuidance(LaneGuidance::Left2Lanes)),
                    "USE_RIGHT_LANE" => Ok(Value::LaneGuidance(LaneGuidance::RightLane)),
                    "USE_RIGHT_2_LANES" => Ok(Value::LaneGuidance(LaneGuidance::Right2Lanes)),
                    "USE_SECOND_FROM_RIGHT" => Ok(Value::LaneGuidance(LaneGuidance::SecondFromRight)),
                    "USE_MIDDLE_LANE" => Ok(Value::LaneGuidance(LaneGuidance::MiddleLane)),
                    "USE_ANY_LANE" => Ok(Value::LaneGuidance(LaneGuidance::AnyLane)),
                    _ => Err(format!("Unexpected value for field 'lane_guidance': {}", symbol))
                }
            }
            ("traffic_light", PF4RawValue::IntValue(index)) => Ok(Value::TrafficLight(TrafficLight { index })),
            ("stop_sign", PF4RawValue::IntValue(index)) => Ok(Value::StopSign(StopSign { index })),
            ("exit_name", PF4RawValue::KeyValueArray(values)) => {
                let mut exits = None;
                for value in values {
                    match value {
                        Value::Exits(r) if exits.is_none() => {
                            exits = Some(r)

                        }
                        value => return Err(format!("Unknown exit_name: {:?}", value))
                    }
                }
                if let Some(exits) = exits {
                    let name = exits.names.iter().map(|n| n.text.as_str()).collect::<Vec<_>>().join("・");
                    Ok(Value::ExitName(ExitName { name }))
                } else {
                    Err("Missing values in exit_name".to_string())
                }
            }
            ("exits", PF4RawValue::KeyValueArray(values)) => {
                let mut names = Vec::new();
                for value in values {
                    if let Value::NLGData(data) = value {
                        names.push(data)
                    } else {
                        return Err(format!("Unknown exits: {:?}", value))
                    }
                }
                Ok(Value::Exits(Exits { names }))
            }
            ("maneuver", PF4RawValue::KeyValueArray(values)) => {
                map_maneuver(values)
            }
            ("key", PF4RawValue::CommandValue(cmd)) => Ok(Value::Key(cmd)),
            ("args", PF4RawValue::KeyValueArray(values)) => Ok(Value::Args(values)),
            ("first_step", PF4RawValue::KeyValueArray(values)) => {
                let [key, args] = values.try_into().map_err(|values| format!("Unknown first_step: {:?}", values))?;
                match (key, args) {
                    (Value::Key(key), Value::Args(args)) =>
                        Ok(Value::FirstStep(map_guidance(key, args)?)),
                    (key, args) => Err(format!("Unknown first_step: {:?} {:?}", key, args))
                }
            }
            ("second_step", PF4RawValue::KeyValueArray(values)) => {
                let [key, args] = values.try_into().map_err(|values| format!("Unknown second_step: {:?}", values))?;
                match (key, args) {
                    (Value::Key(key), Value::Args(args)) =>
                        Ok(Value::SecondStep(map_guidance(key, args)?)),
                    (key, args) => Err(format!("Unknown second_step: {:?} {:?}", key, args))
                }
            }
            ("sign_direct_name", PF4RawValue::KeyValueArray(values)) => {
                let mut routes = None;
                for value in values {
                    match value {
                        Value::Routes(r) if routes.is_none() => {
                            routes = Some(r)

                        }
                        value => return Err(format!("Unknown sign_direct_name: {:?}", value))
                    }
                }
                if let Some(routes) = routes {
                    let name = routes.names.iter().map(|n| n.text.as_str()).collect::<Vec<_>>().join("・");
                    Ok(Value::SignDirectName(name))
                } else {
                    Err("Missing values in sign_direct_name".to_string())
                }
            }
            ("sign_indirect_name", PF4RawValue::KeyValueArray(values)) => {
                let mut routes = None;
                for value in values {
                    match value {
                        Value::Routes(r) if routes.is_none() => {
                            routes = Some(r)

                        }
                        value => return Err(format!("Unknown sign_indirect_name: {:?}", value))
                    }
                }
                if let Some(routes) = routes {
                    let name = routes.names.iter().map(|n| n.text.as_str()).collect::<Vec<_>>().join("・");
                    Ok(Value::SignIndirectName(name))
                } else {
                    Err("Missing values in sign_indirect_name".to_string())
                }
            }
            ("routes", PF4RawValue::KeyValueArray(values)) => {
                let mut names = Vec::new();
                for value in values {
                    if let Value::NLGData(data) = value {
                        names.push(data)
                    } else {
                        return Err(format!("Unknown routes: {:?}", value))
                    }
                }
                Ok(Value::Routes(Routes { names }))
            }
            ("intersection_name", PF4RawValue::KeyValueArray(values)) => {
                let mut routes = None;
                for value in values {
                    match value {
                        Value::Routes(r) if routes.is_none() => {
                            routes = Some(r)

                        }
                        value => return Err(format!("Unknown intersection_name: {:?}", value))
                    }
                }
                if let Some(routes) = routes {
                    let name = routes.names.iter().map(|n| n.text.as_str()).collect::<Vec<_>>().join("・");
                    Ok(Value::IntersectionName(name))
                } else {
                    Err("Missing values in intersection_name".to_string())
                }
            }
            ("interchange_name", PF4RawValue::KeyValueArray(values)) => {
                let mut exits = None;
                for value in values {
                    match value {
                        Value::Exits(r) if exits.is_none() => {
                            exits = Some(r)

                        }
                        value => return Err(format!("Unknown intersection_name: {:?}", value))
                    }
                }
                if let Some(exits) = exits {
                    let name = exits.names.iter().map(|n| n.text.as_str()).collect::<Vec<_>>().join("・");
                    Ok(Value::InterchangeName(InterchangeName { name }))
                } else {
                    Err("Missing values in interchange_name".to_string())
                }
            }
            (_, value) => {
                Err(format!("Unknown key-value pair: {} = {:?}", key, value))
            }
        }
    } else if let PF4RawValue::NLGData(data) = value {
        Ok(Value::NLGData(NLGData { text: data.text }))
    } else {
        Err("Missing key in PF4KeyValue".to_string())
    }
}

fn map_maneuver(values: Vec<Value>) -> Result<Value, String> {
    let [key, args] = values.try_into().map_err(|values| format!("Unknown maneuver: {:?}", values))?;
    match (key, args) {
        (Value::Key(key), Value::Args(args)) =>
            Ok(Value::Maneuver(map_guidance(key, args)?)),
        (key, args) => Err(format!("Unknown maneuver: {:?} {:?}", key, args))
    }
}

fn parse_value(bytes: &[u8]) -> Result<Value, String> {
    let mut key = None;
    let mut raw_value = None;
    let fields = parse_fields(bytes)?;
    for &Field { ref field_number, ref value} in &fields {
        if field_number.as_u32() == 1 {
            if let FieldValue::Len(nested_bytes) = value {
                if key.is_some() {
                    return Err("Duplicate field 1 in PF4KeyValue".to_string())
                } else {
                    key = Some(parse_string(nested_bytes)?)
                }
            } else {
                return Err(format!("Unexpected field type for field 1 in PF4KeyValue: {:?}", value))
            }
        } else {
            let new_raw_value = match (field_number.as_u32(), value) {
                (3, &FieldValue::Varint(v)) => {
                    Some(PF4RawValue::IntValue(v.to_sint64()))
                }
                (4, &FieldValue::Len(nested_bytes)) => {
                    Some(PF4RawValue::SymbolValue(parse_string(nested_bytes)?))
                }
                (6, &FieldValue::I64(nested_bytes)) => {
                    Some(PF4RawValue::FloatValue(f64::from_le_bytes(nested_bytes)))
                }
                (11, &FieldValue::Len(nested_bytes)) => {
                    Some(PF4RawValue::EnumValue(parse_enum(nested_bytes)?))
                }
                (14, &FieldValue::Len(nested_bytes)) => {
                    Some(PF4RawValue::CommandValue(parse_command(nested_bytes)?))
                }
                (15, &FieldValue::Len(nested_bytes)) => {
                    Some(PF4RawValue::NLGData(parse_nlg_data(nested_bytes)?))
                }
                (16, &FieldValue::Len(nested_bytes)) => {
                    Some(PF4RawValue::KeyValueArray(parse_values(nested_bytes)?))
                }
                (n, _) => {
                    return Err(format!("Unexpected field number {} in PF4KeyValue", n))
                }
            };
            if raw_value.is_some() {
                return Err(format!("Duplicate value field in PF4KeyValue: {:?}", raw_value))
            } else {
                raw_value = new_raw_value;
            }
        }
    }
    if let Some(value) = raw_value {
        map_value(key, value)
    } else {
        Err("Missing value in PF4KeyValue".to_string())
    }
}

fn parse_enum(bytes: &[u8]) -> Result<PF4Enum, String> {
    let mut type_name = None;
    let mut enum_value = None;
    let fields = parse_fields(bytes)?;
    for &Field { ref field_number, ref value } in &fields {
            match (field_number.as_u32(), value) {
            (1, &FieldValue::Len(nested_bytes)) if type_name.is_none() => {
                type_name = Some(parse_string(nested_bytes)?);
            }
            (2, &FieldValue::Len(nested_bytes)) if enum_value.is_none() => {
                enum_value = Some(parse_string(nested_bytes)?);
            }
            (n, _) => return Err(format!("Unexpected field number {} in PF4Enum", n))
        }
    }
    if let (Some(type_name), Some(enum_value)) = (type_name, enum_value) {
        Ok(PF4Enum { type_name, value: enum_value })
    } else {
        Err("Missing required fields in PF4Enum".to_string())
    }
}

fn parse_nlg_data(bytes: &[u8]) -> Result<PF4NLGData, String> {
    let mut id = None;
    let mut text = None;
    let fields = parse_fields(bytes)?;
    for &Field { ref field_number, ref value } in &fields {
        match (field_number.as_u32(), value) {
            (1, &FieldValue::Len(nested_bytes)) if id.is_none() => {
                id = Some(parse_string(nested_bytes)?);
            }
            (2, &FieldValue::Len(nested_bytes)) if text.is_none() => {
                text = Some(parse_string(nested_bytes)?);
            }
            (n, _) => return Err(format!("Unexpected field number {} in PF4NLGData", n))
        }
    }
    if let Some(text) = text {
        Ok(PF4NLGData { id, text })
    } else {
        Err("Missing required fields in PF4NLGData".to_string())
    }
}

fn parse_command(bytes: &[u8]) -> Result<PF4Enum, String> {
    let mut command_value = None;
    let fields = parse_fields(bytes)?;
    for &Field { ref field_number, ref value } in &fields {
            match (field_number.as_u32(), value) {
            (3, &FieldValue::Len(nested_bytes)) if command_value.is_none() => {
                command_value = Some(parse_enum(nested_bytes)?);
            }
            (n, _) => return Err(format!("Unexpected field number {} in PF4Command", n))
        }
    }
    if let Some(value) = command_value {
        Ok(value)
    } else {
        Err("Missing required field 3 in PF4Command".to_string())
    }
}

fn parse_fields(bytes: &[u8]) -> Result<Vec<Field<&[u8]>>, String> {
    AsRefExtProtobuf::read_protobuf_fields(bytes)
        .collect::<Result<Vec<Field<&[u8]>>, _>>()
        .map_err(|e| format!("Failed to parse protobuf fields: {}", e))
}

#[allow(dead_code)]
pub fn dump_proto(bytes: &[u8], indent: usize) -> Result<String, String> {
    let mut out = String::new();
    out.push_str(&format!("{}(\n", " ".repeat(indent)));
    let fields = parse_fields(bytes)?;
    for field in &fields {
        match &field.value {
            FieldValue::Len(nested_bytes) => {
                let s = str::from_utf8(nested_bytes);
                match s {
                    Ok(s) if s.chars().all(|c| !c.is_control()) => {
                        out.push_str(&format!("{}{}: {}\n", " ".repeat(indent), field.field_number.as_u32(), s));
                    }
                    _ => {
                        out.push_str(&format!("{}{}: (binary data, length {})\n", " ".repeat(indent), field.field_number.as_u32(), nested_bytes.len()));
                        out.push_str(&dump_proto(nested_bytes, indent + 2)?);
                    }
                }
            }
            other => {
                out.push_str(&format!("{}{}: {:?}\n", " ".repeat(indent), field.field_number.as_u32(), other));
            }
        }
    }
    out.push_str(&format!("{})\n", " ".repeat(indent)));
    Ok(out)
}


#[cfg(test)]
mod tests {
    use super::*;
    use base64::prelude::*;

    const TEST_CASES: &[&str] = &[
        // およそ 500 メートル先、、右側 2 車線を使用して右折する
        concat!(
            "EpQCEuYBChMKCGRpc3RhbmNlMQAAAAAA",
            "QH9ACjcKDWRpc3RhbmNlX3VuaXRaJgoX",
            "bmxwX2dlbmVyYXRpb24uVW5pdFR5cGUS",
            "C1VOSVRfTUVURVJTCpUBCghtYW5ldXZl",
            "coIBhwEKIwoDa2V5chwaGgoLcGF0aGZp",
            "bmRlcjQSC3BmX3R1cm5zdGVwCmAKBGFy",
            "Z3OCAVcKJwoSbGFuZV9ndWlkYW5jZV90",
            "eXBlIhFVU0VfUklHSFRfMl9MQU5FUwoY",
            "Cg50dXJuX3NoYXJwbmVzcyIGTk9STUFM",
            "ChIKCXR1cm5fc2lkZSIFUklHSFQqKRon",
            "CgtwYXRoZmluZGVyNBIYcHJlcGFyZV9k",
            "aXN0YW5jZV9tZXNzYWdl"),
        // およそ 400 メートル先、、右折する
        concat!(
            "EukBErsBChMKCGRpc3RhbmNlMQAAAAAA",
            "AHlACjcKDWRpc3RhbmNlX3VuaXRaJgoX",
            "bmxwX2dlbmVyYXRpb24uVW5pdFR5cGUS",
            "C1VOSVRfTUVURVJTCmsKCG1hbmV1dmVy",
            "ggFeCiMKA2tleXIcGhoKC3BhdGhmaW5k",
            "ZXI0EgtwZl90dXJuc3RlcAo3CgRhcmdz",
            "ggEuChgKDnR1cm5fc2hhcnBuZXNzIgZO",
            "T1JNQUwKEgoJdHVybl9zaWRlIgVSSUdI",
            "VCopGicKC3BhdGhmaW5kZXI0EhhwcmVw",
            "YXJlX2Rpc3RhbmNlX21lc3NhZ2U="),
        // およそ 400 メートル先、、京葉市川インターチェンジ 出口を 国道14号 方面に向かって進みます
        concat!(
            "EqYCEvgBChMKCGRpc3RhbmNlMQAAAAAA",
            "AHlACjcKDWRpc3RhbmNlX3VuaXRaJgoX",
            "bmxwX2dlbmVyYXRpb24uVW5pdFR5cGUS",
            "C1VOSVRfTUVURVJTCqcBCghtYW5ldXZl",
            "coIBmQEKJgoDa2V5ch8aHQoLcGF0aGZp",
            "bmRlcjQSDnBmX29mZnJhbXBzdGVwCm8K",
            "BGFyZ3OCAWYKZAoJZXhpdF9uYW1lggFW",
            "ClQKBWV4aXRzggFKCkh6RgoeL0ZFQVRV",
            "UkVfSUQvMHhmMmQ1ZDAxYjg0YTJjMzUy",
            "EiTkuqzokYnluILlt53jgqTjg7Pjgr/j",
            "g7zjg4Hjgqfjg7PjgrgqKRonCgtwYXRo",
            "ZmluZGVyNBIYcHJlcGFyZV9kaXN0YW5j",
            "ZV9tZXNzYWdl"),
        // 京葉市川インターチェンジ を出ます、続いて 斜め左方向に曲がり 市川 方面のランプにはいります
        concat!(
            "EqYDEvICCqkBCgpmaXJzdF9zdGVwggGZ",
            "AQomCgNrZXlyHxodCgtwYXRoZmluZGVy",
            "NBIOcGZfb2ZmcmFtcHN0ZXAKbwoEYXJn",
            "c4IBZgpkCglleGl0X25hbWWCAVYKVAoF",
            "ZXhpdHOCAUoKSHpGCh4vRkVBVFVSRV9J",
            "RC8weGYyZDVkMDFiODRhMmMzNTISJOS6",
            "rOiRieW4guW3neOCpOODs+OCv+ODvOOD",
            "geOCp+ODs+OCuArDAQoLc2Vjb25kX3N0",
            "ZXCCAbIBCiUKA2tleXIeGhwKC3BhdGhm",
            "aW5kZXI0Eg1wZl9vbnJhbXBzdGVwCogB",
            "CgRhcmdzggF/ChgKDnR1cm5fc2hhcnBu",
            "ZXNzIgZTTElHSFQKEQoJdHVybl9zaWRl",
            "IgRMRUZUClAKEnNpZ25faW5kaXJlY3Rf",
            "bmFtZYIBOQo3CgZyb3V0ZXOCASwKKnoo",
            "Ch4vRkVBVFVSRV9JRC8weGZmMjBlOTlh",
            "ZjkxYWM1MzkSBuW4guW3nSovGi0KC3Bh",
            "dGhmaW5kZXI0Eh5jb21iaW5lX21lcmdl",
            "ZF9ndWlkYW5jZV9ldmVudHM="),
        // 斜め左方向に曲がり 市川 方面のランプにはいります、続いて 左折する
        concat!(
            "EugCErQCCsIBCgpmaXJzdF9zdGVwggGy",
            "AQolCgNrZXlyHhocCgtwYXRoZmluZGVy",
            "NBINcGZfb25yYW1wc3RlcAqIAQoEYXJn",
            "c4IBfwoYCg50dXJuX3NoYXJwbmVzcyIG",
            "U0xJR0hUChEKCXR1cm5fc2lkZSIETEVG",
            "VApQChJzaWduX2luZGlyZWN0X25hbWWC",
            "ATkKNwoGcm91dGVzggEsCip6KAoeL0ZF",
            "QVRVUkVfSUQvMHhmZjIwZTk5YWY5MWFj",
            "NTM5EgbluILlt50KbQoLc2Vjb25kX3N0",
            "ZXCCAV0KIwoDa2V5chwaGgoLcGF0aGZp",
            "bmRlcjQSC3BmX3R1cm5zdGVwCjYKBGFy",
            "Z3OCAS0KGAoOdHVybl9zaGFycG5lc3Mi",
            "Bk5PUk1BTAoRCgl0dXJuX3NpZGUiBExF",
            "RlQqLxotCgtwYXRoZmluZGVyNBIeY29t",
            "YmluZV9tZXJnZWRfZ3VpZGFuY2VfZXZl",
            "bnRz"),
        // 信号を左方向です
        concat!(
            "EmASQAoRCg10cmFmZmljX2xpZ2h0GAEK",
            "GAoOdHVybl9zaGFycG5lc3MiBk5PUk1B",
            "TAoRCgl0dXJuX3NpZGUiBExFRlQqHBoa",
            "CgtwYXRoZmluZGVyNBILcGZfdHVybnN0",
            "ZXA="),
        // そのまま 1 キロ進みます
        concat!(
            "EnwSUgoTCghkaXN0YW5jZTEAAAAAAADw",
            "Pwo7Cg1kaXN0YW5jZV91bml0WioKF25s",
            "cF9nZW5lcmF0aW9uLlVuaXRUeXBlEg9V",
            "TklUX0tJTE9NRVRFUlMqJhokCgtwYXRo",
            "ZmluZGVyNBIVY29udGludWVfZm9yX2Rp",
            "c3RhbmNl"),
        // およそ 300 メートル先、、右方向です
        concat!(
            "EukBErsBChMKCGRpc3RhbmNlMQAAAAAA",
            "wHJACjcKDWRpc3RhbmNlX3VuaXRaJgoX",
            "bmxwX2dlbmVyYXRpb24uVW5pdFR5cGUS",
            "C1VOSVRfTUVURVJTCmsKCG1hbmV1dmVy",
            "ggFeCiMKA2tleXIcGhoKC3BhdGhmaW5k",
            "ZXI0EgtwZl90dXJuc3RlcAo3CgRhcmdz",
            "ggEuChgKDnR1cm5fc2hhcnBuZXNzIgZO",
            "T1JNQUwKEgoJdHVybl9zaWRlIgVSSUdI",
            "VCopGicKC3BhdGhmaW5kZXI0EhhwcmVw",
            "YXJlX2Rpc3RhbmNlX21lc3NhZ2U="),
        // 右方向です
        concat!(
            "Ek4SLgoYCg50dXJuX3NoYXJwbmVzcyIG",
            "Tk9STUFMChIKCXR1cm5fc2lkZSIFUklH",
            "SFQqHBoaCgtwYXRoZmluZGVyNBILcGZf",
            "dHVybnN0ZXA="),
        // およそ 200 メートル先、、目的地です。
        concat!(
            "EsoBEpwBChMKCGRpc3RhbmNlMQAAAAAA",
            "AGlACjcKDWRpc3RhbmNlX3VuaXRaJgoX",
            "bmxwX2dlbmVyYXRpb24uVW5pdFR5cGUS",
            "C1VOSVRfTUVURVJTCkwKCG1hbmV1dmVy",
            "ggE/CjIKA2tleXIrGikKC3BhdGhmaW5k",
            "ZXI0EhpwZl9kZXN0aW5hdGlvbnN0ZXBf",
            "cHJlcGFyZQoJCgRhcmdzggEAKikaJwoL",
            "cGF0aGZpbmRlcjQSGHByZXBhcmVfZGlz",
            "dGFuY2VfbWVzc2FnZQ=="),
        // 目的地に到着しました。
        concat!(
            "EisSAConGiUKC3BhdGhmaW5kZXI0EhZw",
            "Zl9kZXN0aW5hdGlvbnN0ZXBfYWN0"),
    ];

    fn parse_and_warn(s64: &str) {
        let s = BASE64_STANDARD.decode(s64).expect("failed to decode base64");
        let dump = dump_proto(&s, 0).unwrap_or_else(|e| format!("failed to dump proto: {}", e));
        let result = parse(&s);
        assert!(result.is_ok(), "parse failed for {}: {:?}\nDump:\n{}", s64, result, dump);
    }

    #[test]
    fn test_parser() {
        for s in TEST_CASES {
            parse_and_warn(s);
        }
    }

    #[test]
    fn test_testdata_files() {
        use std::path::Path;
        let dir = Path::new("testdata");
        if !dir.exists() {
            eprintln!("testdata directory not found, skipping");
            return;
        }
        for entry in std::fs::read_dir(dir).expect("failed to read testdata dir") {
            let entry = entry.expect("failed to read dir entry");
            let path = entry.path();
            if path.is_file() {
                let content = std::fs::read_to_string(&path).expect("failed to read test file");
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() { continue; }
                    let b64 = if let Some(i) = line.rfind(',') { &line[i+1..] } else { line };
                    parse_and_warn(b64);
                }
            }
        }
    }
}

