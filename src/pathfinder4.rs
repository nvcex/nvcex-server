use protobuf_core::{AsRefExtProtobuf, Field, FieldValue, WireType::Varint};
use std::str;

#[derive(Debug)]
pub enum Guidance {
    StraightStep(Option<LaneGuidance>),
    TurnStep(TurnSharpness, TurnSide, Option<LaneGuidance>, Option<IntersectionName>, Option<TrafficLight>),
    UTurnStep(Option<IntersectionName>),
    OnRampStep(Option<TurnSharpness>, Option<TurnSide>, Option<LaneGuidance>, Option<IntersectionName>, Option<TrafficLight>, Option<SignIndirectName>),
    OffRampStep(Option<LaneGuidance>, Option<ExitName>, Option<SignIndirectName>),
    KeepOrForkStep(KeepSide, Option<LaneGuidance>),
    MergeStep(Option<LaneGuidance>),
    InterchangeStep,
    DestinationStepPrepare,
    DestinationStepAct,
    ContinueForDistance,
    PrepareDistanceMessage,
    CombineMergedGuidanceEvents,    
}

#[derive(Debug, Clone, Copy)]
pub enum DistanceUnit {
    Meter,
    Kilometer,
    Mile,
}

#[derive(Debug, Clone, Copy)]
pub enum TurnSharpness {
    Slight,
    Normal,
    Sharp,
}

#[derive(Debug, Clone, Copy)]
pub enum TurnSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum KeepSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum DestinationSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum LaneGuidance {
    LeftLane,
    Left2Lanes,
    MiddleLane,
    RightLane,
    Right2Lanes,
    SecondFromRight,
    AnyLane
}

#[derive(Debug)]
pub struct TrafficLight {
    index: i64
}

#[derive(Debug)]
pub struct IntersectionName {
    routes: Routes
}

#[derive(Debug)]
pub struct InterchangeName {
    exits: Exits
}

#[derive(Debug)]
pub struct ExitName {
    exits: Exits
}

#[derive(Debug)]
pub struct Exits {
    names: Vec<NLGData>
}

#[derive(Debug)]
pub struct SignDirectName {
    routes: Routes
}

#[derive(Debug)]
pub struct SignIndirectName {
    routes: Routes
}

#[derive(Debug)]
pub struct Routes {
    names: Vec<NLGData>
}

#[derive(Debug)]
pub struct NLGData {
    text: String
}

#[derive(Debug)]
enum Value {
    Unknown(String, String),
    Distance(f64),
    DistanceUnit(DistanceUnit),
    DistanceOverride(String),
    TurnSharpness(TurnSharpness),
    TurnSide(TurnSide),
    KeepSide(KeepSide),
    DestinationSide(DestinationSide),
    LaneGuidance(LaneGuidance),
    TrafficLight(TrafficLight),
    StopSign(i64),
    ExitName(ExitName),
    Exits(Exits),
    Maneuver(Guidance),
    Key(PF4Enum),
    Args(Vec<Value>),
    FirstStep(Vec<Value>),
    SecondStep(Vec<Value>),
    SignDirectName(SignDirectName),
    SignIndirectName(SignIndirectName),
    Routes(Routes),
    IntersectionName(IntersectionName),
    InterchangeName(InterchangeName),
    NLGData(NLGData),
}

#[derive(Debug)]
enum PF4RawValue {
    // field 2: string
    StringValue(String),
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
    PF4Unknown,
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
    pub id: Option<String>,
    // field 2: string
    pub text: String,
}

pub struct Parser {
    pub warnings: Vec<String>,
}

impl Parser {
    pub fn new() -> Self {
        Parser { warnings: Vec::new() }
    }

    fn warn(&mut self, message: String) {
        self.warnings.push(message);
    }

    pub fn parse(&mut self, bytes: &[u8]) -> Result<Guidance, String> {
        let mut body = None;
        let fields = parse_fields(bytes)?;
        for &Field { ref field_number, ref value } in &fields {
            match (field_number.as_u32(), value) {
                (2, &FieldValue::Len(nested_bytes)) if body.is_none()=> {
                    body = Some(self.parse_guidance(nested_bytes)?);
                }
                (n, _) => {
                    self.warn(format!("Unexpected field number {} in envelope", n));
                }
            }
        }
        if let Some(body) = body {
            Ok(body)
        } else {
            Err("Missing required field 2 in StructuredMessage".to_string())
        }
    }

    fn parse_string(&mut self, bytes: &[u8]) -> Result<String, String> {
        match str::from_utf8(bytes) {
            Ok(s) => Ok(s.to_string()),
            Err(e) => Err(format!("invalid utf-8 sequence: {}", e)),
        }
    }

    fn parse_guidance(&mut self, bytes: &[u8]) -> Result<Guidance, String> {
        let mut args = None;
        let mut command = None;
        let fields = parse_fields(bytes)?;
        for &Field { ref field_number, ref value } in &fields {
             match (field_number.as_u32(), value) {
                (2, &FieldValue::Len(nested_bytes)) if args.is_none() => {
                    args = Some(self.parse_values(nested_bytes)?);
                }
                (5, &FieldValue::Len(nested_bytes)) if command.is_none() => {
                    command = Some(self.parse_command(nested_bytes)?);
                }
                (n, _) => {
                    self.warn(format!("Unexpected field number {} in PF4Message", n));
                }
            }
        }
        if let (Some(args), Some(command)) = (args, command) {
            self.map_guidance(command, args)
        } else {
            Err("Missing required fields in PF4Message".to_string())
        }
    }

    fn map_guidance(&mut self, command: PF4Enum, args: Vec<Value>) -> Result<Guidance, String> {
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
                    let mut intersectionName = None;
                    let mut trafficLight = None;
                    for arg in args {
                        match arg {
                            Value::LaneGuidance(v) => lane = Some(v),
                            Value::TurnSharpness(v) => sharpness = Some(v),
                            Value::TurnSide(v) => side = Some(v),
                            Value::IntersectionName(v) => intersectionName = Some(v),
                            Value::TrafficLight(v) => trafficLight = Some(v),
                            arg => return Err(format!("unknown args for pf_turnstep {:?}", arg))
                        }
                    }
                    if let (Some(sharpness), Some(side)) = (sharpness, side) {
                        Ok(Guidance::TurnStep(sharpness, side, lane, intersectionName, trafficLight))
                    } else {
                        Err("missing sharpness or side field in pf_turnstep".to_string())
                    }
                }
                "pf_uturnstep" => {
                    let mut intersectionName = None;
                    for arg in args {
                        match arg {
                            Value::IntersectionName(v) => intersectionName = Some(v),
                            arg => return Err(format!("unknown args for pf_uturnstep {:?}", arg))
                        }
                    }
                    Ok(Guidance::UTurnStep(intersectionName))
                }
                "pf_onrampstep" => {
                    let mut lane = None;
                    let mut sharpness = None;
                    let mut side = None;
                    let mut intersectionName = None;
                    let mut trafficLight = None;
                    let mut signName = None;
                    for arg in args {
                        match arg {
                            Value::LaneGuidance(v) => lane = Some(v),
                            Value::TurnSharpness(v) => sharpness = Some(v),
                            Value::TurnSide(v) => side = Some(v),
                            Value::IntersectionName(v) => intersectionName = Some(v),
                            Value::TrafficLight(v) => trafficLight = Some(v),
                            Value::SignIndirectName(v) => signName = Some(v),
                            arg => return Err(format!("unknown args for pf_onrampstep {:?}", arg))
                        }
                    }
                    Ok(Guidance::OnRampStep(sharpness, side, lane, intersectionName, trafficLight, signName))
                }
                "pf_offrampstep" => {
                    let mut lane = None;
                    let mut exit = None;
                    let mut signName = None;
                    for arg in args {
                        match arg {
                            Value::LaneGuidance(v) => lane = Some(v),
                            Value::ExitName(v) => exit = Some(v),
                            Value::SignIndirectName(v) => signName = Some(v),
                            arg => return Err(format!("unknown args for pf_offrampstep {:?}", arg))
                        }
                    }
                    Ok(Guidance::OffRampStep(lane, exit, signName))
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
                    let mut name = None;
                    let mut signDirectName = None;
                    let mut signIndirectName = None;
                    for arg in args {
                        match arg {
                            Value::InterchangeName(v) => name = Some(v),
                            Value::SignDirectName(v) => signDirectName = Some(v),
                            Value::SignIndirectName(v) => signIndirectName = Some(v),
                            arg => return Err(format!("unknown args for pf_offrampstep {:?}", arg))
                        }
                    }
                    if let Some(name) = name {
                        Ok(Guidance::InterchangeStep)
                    } else {
                        Err("Missing field in pf_interchangestep".to_string())
                    }
                }
                "pf_destinationstep_prepare" => match args.as_slice() {
                    [] => Ok(Guidance::DestinationStepPrepare),
                    values => Err(format!("unknown args for pf_destinationstep_prepare {:?}", values))
                }
                "pf_destinationstep_act" => match args.as_slice() {
                    [] => Ok(Guidance::DestinationStepAct),
                    values => Err(format!("unknown args for pf_destinationstep_act {:?}", values))
                }
                "continue_for_distance" => match args.as_slice() {
                    [Value::Distance(_), Value::DistanceUnit(_)] => Ok(Guidance::ContinueForDistance),
                    [Value::Distance(_), Value::DistanceUnit(_), Value::DistanceOverride(_)] => Ok(Guidance::ContinueForDistance),
                    values => Err(format!("unknown args for continue_for_distance {:?}", values))
                }
                "prepare_distance_message" => match args.as_slice() {
                    [Value::Distance(_), Value::DistanceUnit(_), Value::Maneuver(_)] => Ok(Guidance::PrepareDistanceMessage),
                    values => Err(format!("unknown args for prepare_distance_message {:?}", values))
                }
                "combine_merged_guidance_events" => match args.as_slice() {
                    [Value::FirstStep(_), Value::SecondStep(_)] => Ok(Guidance::CombineMergedGuidanceEvents),
                    values => Err(format!("unknown args for combine_merged_guidance_events {:?}", values))
                }
                v => Err(format!("Unknown command value {}", v))
            }
        } else {
            Err(format!("Unknown command type {}", command.type_name))
        }
    }

    fn parse_values(&mut self, bytes: &[u8]) -> Result<Vec<Value>, String> {
        let mut values = Vec::new();
        let fields = parse_fields(bytes)?;
        for &Field { ref field_number, ref value } in &fields {
            match (field_number.as_u32(), value) {
                (1, &FieldValue::Len(nested_bytes)) => {
                    values.push(self.parse_value(nested_bytes)?);
                }
                (n, _) => {
                    self.warn(format!("Unexpected field number {} in PF4MessageFields", n));
                }
            }
        }
        Ok(values)
    }

    fn map_value(&mut self, key: Option<String>, value: PF4RawValue) -> Result<Value, String> {
        if let Some(key) = key {
            match (key.as_str(), value) {
                ("distance", PF4RawValue::FloatValue(f)) => Ok(Value::Distance(f)),
                ("distance_unit", PF4RawValue::EnumValue(enum_value)) if enum_value.type_name == "nlp_generation.UnitType" => {
                    match enum_value.value.as_str() {
                        "UNIT_METERS" => Ok(Value::DistanceUnit(DistanceUnit::Meter)),
                        "UNIT_KILOMETERS" => Ok(Value::DistanceUnit(DistanceUnit::Kilometer)),
                        "UNIT_MILES" => Ok(Value::DistanceUnit(DistanceUnit::Mile)),
                        _ => {
                            self.warnings.push(format!("Unexpected value for field 'distance_unit': {}", enum_value.value));
                            Ok(Value::Unknown(key, enum_value.value.clone()))
                        }
                    }
                }
                ("distance_override_type", PF4RawValue::SymbolValue(symbol)) => Ok(Value::DistanceOverride(symbol)),
                ("turn_sharpness", PF4RawValue::SymbolValue(symbol)) => {
                    match symbol.as_str() {
                        "SLIGHT" => Ok(Value::TurnSharpness(TurnSharpness::Slight)),
                        "NORMAL" => Ok(Value::TurnSharpness(TurnSharpness::Normal)),
                        "SHARP" => Ok(Value::TurnSharpness(TurnSharpness::Sharp)),
                        _ => {
                            self.warnings.push(format!("Unexpected value for field 'turn_sharpness': {}", symbol));
                            Ok(Value::Unknown(key, symbol))
                        }
                    }
                }
                ("turn_side", PF4RawValue::SymbolValue(symbol)) => {
                    match symbol.as_str() {
                        "LEFT" => Ok(Value::TurnSide(TurnSide::Left)),
                        "RIGHT" => Ok(Value::TurnSide(TurnSide::Right)),
                        _ => {
                            self.warnings.push(format!("Unexpected value for field 'turn_side': {}", symbol));
                            Ok(Value::Unknown(key, symbol.clone()))
                        }
                    }
                }
                ("keep_side", PF4RawValue::SymbolValue(symbol)) => {
                    match symbol.as_str() {
                        "LEFT" => Ok(Value::KeepSide(KeepSide::Left)),
                        "RIGHT" => Ok(Value::KeepSide(KeepSide::Right)),
                        _ => {
                            self.warnings.push(format!("Unexpected value for field 'keep_side': {}", symbol));
                            Ok(Value::Unknown(key, symbol.clone()))
                        }
                    }
                }
                ("destination_side", PF4RawValue::SymbolValue(symbol)) => {
                    match symbol.as_str() {
                        "LEFT" => Ok(Value::DestinationSide(DestinationSide::Left)),
                        "RIGHT" => Ok(Value::DestinationSide(DestinationSide::Right)),
                        _ => {
                            self.warnings.push(format!("Unexpected value for field 'destination_side': {}", symbol));
                            Ok(Value::Unknown(key, symbol.clone()))
                        }
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
                        _ => {
                            self.warnings.push(format!("Unexpected value for field 'lane_guidance': {}", symbol));
                            Ok(Value::Unknown(key, symbol.clone()))
                        }
                    }
                }
                ("traffic_light", PF4RawValue::IntValue(index)) => Ok(Value::TrafficLight(TrafficLight { index })),
                ("stop_sign", PF4RawValue::IntValue(v)) => Ok(Value::StopSign(v)),
                ("exit_name", PF4RawValue::KeyValueArray(values)) => {
                    let mut exits = None;
                    for value in values {
                        match value {
                            Value::Exits(r) if exits.is_none() => {
                                exits = Some(r)

                            }
                            value => {
                                self.warnings.push(format!("Unknown intersection_name: {:?}", value));
                            }
                        }
                    }
                    if let Some(exits) = exits {
                        Ok(Value::ExitName(ExitName { exits }))
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
                            self.warnings.push(format!("Unknown routes: {:?}", value));
                        }
                    }
                    Ok(Value::Exits(Exits { names }))
                }
                ("maneuver", PF4RawValue::KeyValueArray(values)) => {
                    self.map_maneuver(values)
                }
                ("key", PF4RawValue::CommandValue(cmd)) => Ok(Value::Key(cmd)),
                ("args", PF4RawValue::KeyValueArray(values)) => Ok(Value::Args(values)),
                ("first_step", PF4RawValue::KeyValueArray(values)) => Ok(Value::FirstStep(values)),
                ("second_step", PF4RawValue::KeyValueArray(values)) => Ok(Value::SecondStep(values)),
                ("sign_direct_name", PF4RawValue::KeyValueArray(values)) => {
                    let mut routes = None;
                    for value in values {
                        match value {
                            Value::Routes(r) if routes.is_none() => {
                                routes = Some(r)

                            }
                            value => {
                                self.warnings.push(format!("Unknown sign_direct_name: {:?}", value));
                            }
                        }
                    }
                    if let Some(routes) = routes {
                        Ok(Value::SignDirectName(SignDirectName { routes }))
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
                            value => {
                                self.warnings.push(format!("Unknown intersection_name: {:?}", value));
                            }
                        }
                    }
                    if let Some(routes) = routes {
                        Ok(Value::SignIndirectName(SignIndirectName { routes }))
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
                            self.warnings.push(format!("Unknown routes: {:?}", value));
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
                            value => {
                                self.warnings.push(format!("Unknown intersection_name: {:?}", value));
                            }
                        }
                    }
                    if let Some(routes) = routes {
                        Ok(Value::IntersectionName(IntersectionName { routes }))
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
                            value => {
                                self.warnings.push(format!("Unknown intersection_name: {:?}", value));
                            }
                        }
                    }
                    if let Some(exits) = exits {
                        Ok(Value::InterchangeName(InterchangeName { exits }))
                    } else {
                        Err("Missing values in interchange_name".to_string())
                    }
                }
                (_, value) => {
                    self.warnings.push(format!("Unknown key-value pair: {} = {:?}", key, value));
                    Ok(Value::Unknown(key, format!("{:?}", value)))
                }
            }
        } else if let PF4RawValue::NLGData(data) = value {
            Ok(Value::NLGData(NLGData { text: data.text }))
        } else {
            Err("Missing key in PF4KeyValue".to_string())
        }
    }

    fn map_maneuver(&mut self, values: Vec<Value>) -> Result<Value, String> {
        let [key, args] = values.try_into().map_err(|values| format!("Unknown maneuver: {:?}", values))?;
        match (key, args) {
            (Value::Key(key), Value::Args(args)) =>
                Ok(Value::Maneuver(self.map_guidance(key, args)?)),
            (key, args) => Err(format!("Unknown maneuver: {:?} {:?}", key, args))
        }
    }

    fn parse_value(&mut self, bytes: &[u8]) -> Result<Value, String> {
        let mut key = None;
        let mut raw_value = None;
        let fields = parse_fields(bytes)?;
        for &Field { ref field_number, ref value} in &fields {
            if field_number.as_u32() == 1 {
                if let FieldValue::Len(nested_bytes) = value {
                    if key.is_some() {
                        self.warnings.push("Duplicate field 1 in PF4KeyValue".to_string());
                    } else {
                        key = Some(self.parse_string(nested_bytes)?);
                    }
                } else {
                    self.warnings.push(format!("Unexpected field type for field 1 in PF4KeyValue: {:?}", value));
                }
            } else {
                let new_raw_value = match (field_number.as_u32(), value) {
                    (2, &FieldValue::Len(nested_bytes)) => {
                        Some(PF4RawValue::StringValue(self.parse_string(nested_bytes)?))
                    }
                    (3, &FieldValue::Varint(v)) => {
                        Some(PF4RawValue::IntValue(v.to_sint64()))
                    }
                    (4, &FieldValue::Len(nested_bytes)) => {
                        Some(PF4RawValue::SymbolValue(self.parse_string(nested_bytes)?))
                    }
                    (6, &FieldValue::I64(nested_bytes)) => {
                        Some(PF4RawValue::FloatValue(f64::from_le_bytes(nested_bytes)))
                    }
                    (11, &FieldValue::Len(nested_bytes)) => {
                        Some(PF4RawValue::EnumValue(self.parse_enum(nested_bytes)?))
                    }
                    (14, &FieldValue::Len(nested_bytes)) => {
                        Some(PF4RawValue::CommandValue(self.parse_command(nested_bytes)?))
                    }
                    (15, &FieldValue::Len(nested_bytes)) => {
                        Some(PF4RawValue::NLGData(self.parse_nlg_data(nested_bytes)?))
                    }
                    (16, &FieldValue::Len(nested_bytes)) => {
                        Some(PF4RawValue::KeyValueArray(self.parse_values(nested_bytes)?))
                    }
                    (n, _) => {
                        self.warn(format!("Unexpected field number {} in PF4KeyValue", n));
                        None
                    }
                };
                if raw_value.is_some() {
                    self.warnings.push(format!("Duplicate value field in PF4KeyValue: {:?}", raw_value));
                } else {
                    raw_value = new_raw_value;
                }
            }
        }
        if let Some(value) = raw_value {
            self.map_value(key, value)
        } else {
            Err("Missing value in PF4KeyValue".to_string())
        }
    }

    fn parse_enum(&mut self, bytes: &[u8]) -> Result<PF4Enum, String> {
        let mut type_name = None;
        let mut enum_value = None;
        let fields = parse_fields(bytes)?;
        for &Field { ref field_number, ref value } in &fields {
             match (field_number.as_u32(), value) {
                (1, &FieldValue::Len(nested_bytes)) if type_name.is_none() => {
                    type_name = Some(self.parse_string(nested_bytes)?);
                }
                (2, &FieldValue::Len(nested_bytes)) if enum_value.is_none() => {
                    enum_value = Some(self.parse_string(nested_bytes)?);
                }
                (n, _) => {
                    self.warn(format!("Unexpected field number {} in PF4Enum", n));
                }
            }
        }
        if let (Some(type_name), Some(enum_value)) = (type_name, enum_value) {
            Ok(PF4Enum { type_name, value: enum_value })
        } else {
            Err("Missing required fields in PF4Enum".to_string())
        }
    }

    fn parse_nlg_data(&mut self, bytes: &[u8]) -> Result<PF4NLGData, String> {
        let mut id = None;
        let mut text = None;
        let fields = parse_fields(bytes)?;
        for &Field { ref field_number, ref value } in &fields {
            match (field_number.as_u32(), value) {
                (1, &FieldValue::Len(nested_bytes)) if id.is_none() => {
                    id = Some(self.parse_string(nested_bytes)?);
                }
                (2, &FieldValue::Len(nested_bytes)) if text.is_none() => {
                    text = Some(self.parse_string(nested_bytes)?);
                }
                (n, _) => {
                    self.warn(format!("Unexpected field number {} in PF4NLGData", n));
                }
            }
        }
        if let Some(text) = text {
            Ok(PF4NLGData { id, text })
        } else {
            Err("Missing required fields in PF4NLGData".to_string())
        }
    }

    fn parse_command(&mut self, bytes: &[u8]) -> Result<PF4Enum, String> {
        let mut command_value = None;
        let fields = parse_fields(bytes)?;
        for &Field { ref field_number, ref value } in &fields {
             match (field_number.as_u32(), value) {
                (3, &FieldValue::Len(nested_bytes)) if command_value.is_none() => {
                    command_value = Some(self.parse_enum(nested_bytes)?);
                }
                (n, _) => {
                    self.warn(format!("Unexpected field number {} in PF4Command", n));
                }
            }
        }
        if let Some(value) = command_value {
            Ok(value)
        } else {
            Err("Missing required field 3 in PF4Command".to_string())
        }
    }
}

fn parse_fields(bytes: &[u8]) -> Result<Vec<Field<&[u8]>>, String> {
    AsRefExtProtobuf::read_protobuf_fields(bytes)
        .collect::<Result<Vec<Field<&[u8]>>, _>>()
        .map_err(|e| format!("Failed to parse protobuf fields: {}", e))
}

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
        let mut parser = Parser::new();
        let result = parser.parse(&s);
        assert!(result.is_ok(), "parse failed for {}: {:?}\nDump:\n{}", s64, result, dump);
        let warnings = parser.warnings;
        assert!(warnings.is_empty(), "warnings for {}: {:?}\nDump:\n{}", s64, warnings, dump);
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

