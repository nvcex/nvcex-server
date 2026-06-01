use base64::prelude::*;
use protobuf_core::{Field, FieldValue, IteratorExtProtobuf, AsRefExtProtobuf};
use std::str;

pub fn parse_fields(bytes: &[u8]) -> Result<Vec<Field<&[u8]>>, String> {
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

#[derive(Debug)]
pub struct StructuredMessage {
    // field 2: PF4Message
    pub body: PF4Message,
}

#[derive(Debug)]
pub struct PF4Message {
    // field 2: PF4MessageFields
    pub fields: PF4MessageFields,
    // field 5: PF4Command
    pub command: PF4Command,
}

pub type PF4MessageFields = Vec<PF4KeyValue>;

#[derive(Debug)]
pub struct PF4Command {
    // field 3: PF4KeyValue
    pub value: PF4KeyValue,
}

#[derive(Debug)]
pub enum PF4Value {
    // field 2: string
    StringValue(String),
    // field 4: Symbol
    SymbolValue(String),
    // field 6: f64
    FloatValue(f64),
    // field 11: PF4Enum
    EnumValue(PF4Enum),
    // field 14: PF4Command
    CommandValue(Box<PF4Command>),
    // field 15: PF4NLGData
    NLGData(PF4NLGData),
    // field 16: PF4KeyValue
    KeyValueArray(Vec<PF4KeyValue>),
    Unknown,
}

#[derive(Debug)]
pub struct PF4Enum {
    // field 1: string
    pub type_name: String,
    // field 2: string
    pub value: String,
}

#[derive(Debug)]
pub struct PF4NLGData {
    // field 1: string
    pub id: String,
    // field 2: string
    pub text: String,
}

#[derive(Debug)]
pub struct PF4KeyValue {
    // field 1: string
    pub key: String,
    pub value: PF4Value,
}

pub struct Parser {
    pub warnings: Vec<String>,
}

impl Parser {
    pub fn new() -> Self {
        Parser { warnings: Vec::new() }
    }

    pub fn parse(&mut self, bytes: &[u8]) -> Result<StructuredMessage, String> {
        let mut body = None;
        let fields = parse_fields(bytes)?;
        for field in &fields {
            if let Some(result) = self.nested_object(field, 2, Self::parse_pf4_message) {
                if body.is_some() {
                    self.warnings.push("Duplicate field 2 in StructuredMessage".to_string());
                } else {
                    body = Some(result?);
                }
            } else {
                self.warnings.push(format!("Unexpected field {:?} in StructuredMessage", field));
            }
        }
        if let Some(body) = body {
            Ok(StructuredMessage { body })
        } else {
            Err("Missing required field 2 in StructuredMessage".to_string())
        }
    }

    fn nested_object<T>(&mut self, field: &Field<&[u8]>, number: u32, body: fn(&mut Self, &[u8]) -> Result<T, String>) -> Option<Result<T, String>> {
        if field.field_number.as_u32() == number {
            if let FieldValue::Len(nested_bytes) = field.value {
                return Some(body(self, nested_bytes));
            }
        }
        None
    }

    fn nested_string(&mut self, field: &Field<&[u8]>, number: u32) -> Option<Result<String, String>> {
        if field.field_number.as_u32() == number {
            if let FieldValue::Len(nested_bytes) = field.value {
                return Some(crate::bytes_to_utf8_string(nested_bytes));
            }
        }
        None
    }

    fn parse_pf4_message(&mut self, bytes: &[u8]) -> Result<PF4Message, String> {
        let mut message_fields = None;
        let mut command = None;
        let fields = parse_fields(bytes)?;
        for field in &fields {
            if let Some(result) = self.nested_object(field, 2, Self::parse_fields) {
                if message_fields.is_some() {
                    self.warnings.push("Duplicate field 1 in PF4Message".to_string());
                } else {
                    message_fields = Some(result?);
                }
            } else if let Some(result) = self.nested_object(field, 5, Self::parse_command) {
                if command.is_some() {
                    self.warnings.push("Duplicate field 2 in PF4Message".to_string());
                } else {
                    command = Some(result?);
                }
            } else {
                self.warnings.push(format!("Unexpected field {:?} in PF4Message", field));
            }
        }
        if let (Some(fields), Some(command)) = (message_fields, command) {
            Ok(PF4Message { fields, command })
        } else {
            Err("Missing required fields in PF4Message".to_string())
        }
    }

    fn parse_fields(&mut self, bytes: &[u8]) -> Result<PF4MessageFields, String> {
        let mut message_fields = Vec::new();
        let fields = parse_fields(bytes)?;
        for field in &fields {
            if let Some(result) = self.nested_object(field, 1, Self::parse_key_value) {
                message_fields.push(result?);
            } else {
                self.warnings.push(format!("Unexpected field {:?} in PF4MessageFields", field));
            }
        }
        Ok(message_fields)
    }

    fn parse_key_value(&mut self, bytes: &[u8]) -> Result<PF4KeyValue, String> {
        let mut key = None;
        let mut value = None;
        let fields = parse_fields(bytes)?;
        for field in &fields {
            if let Some(result) = self.nested_string(field, 1) {
                if key.is_some() {
                    self.warnings.push("Duplicate field 1 in PF4KeyValue".to_string());
                } else {
                    key = Some(result?);
                }
            } else if let Some(result) = self.nested_string(field, 2) {
                if value.is_some() {
                    self.warnings.push("Duplicate field 2 in PF4KeyValue".to_string());
                } else {
                    value = Some(PF4Value::StringValue(result?));
                }
            } else if let Some(result) = self.nested_string(field, 4) {
                if value.is_some() {
                    self.warnings.push("Duplicate field 4 in PF4KeyValue".to_string());
                } else {
                    value = Some(PF4Value::SymbolValue(result?));
                }
            } else if field.field_number.as_u32() == 6 {
                if let FieldValue::I64(nested_bytes) = field.value {
                    if value.is_some() {
                        self.warnings.push("Duplicate field 6 in PF4KeyValue".to_string());
                    } else {
                        let f = f64::from_le_bytes(nested_bytes);
                        value = Some(PF4Value::FloatValue(f));
                    }
                } else {
                    self.warnings.push(format!("Unexpected field type for field 6 in PF4KeyValue: {:?}", field.value));
                }
            } else if let Some(result) = self.nested_object(field, 11, Self::parse_enum) {
                if value.is_some() {
                    self.warnings.push("Duplicate field 11 in PF4KeyValue".to_string());
                } else {
                    value = Some(PF4Value::EnumValue(result?));
                }
            } else if let Some(result) = self.nested_object(field, 14, Self::parse_command) {
                if value.is_some() {
                    self.warnings.push("Duplicate field 14 in PF4KeyValue".to_string());
                } else {
                    value = Some(PF4Value::CommandValue(Box::new(result?)));
                }
            } else if let Some(result) = self.nested_object(field, 15, Self::parse_nlg_data) {
                if value.is_some() {
                    self.warnings.push("Duplicate field 15 in PF4KeyValue".to_string());
                } else {
                    value = Some(PF4Value::NLGData(result?));
                }
            } else if let Some(result) = self.nested_object(field, 16, Self::parse_key_value_array) {
                if value.is_some() {
                    self.warnings.push("Duplicate field 16 in PF4KeyValue".to_string());
                } else {
                    value = Some(PF4Value::KeyValueArray(result?));
                }
             } else {
                 self.warnings.push(format!("Unexpected field {:?} in PF4KeyValue", field));
             }
        }
        if let Some(key) = key {
            Ok(PF4KeyValue { key, value: value.unwrap_or(PF4Value::Unknown) })
        } else if let Some(PF4Value::NLGData(_)) = value {
            Ok(PF4KeyValue { key: "<nlg_data>".to_string(), value: value.unwrap() })
        } else {
            Err("Missing required fields in PF4KeyValue".to_string())
        }
    }

    fn parse_enum(&mut self, bytes: &[u8]) -> Result<PF4Enum, String> {
        let mut type_name = None;
        let mut value = None;
        let fields = parse_fields(bytes)?;
        for field in &fields {
            if let Some(result) = self.nested_string(field, 1) {
                if type_name.is_some() {
                    self.warnings.push("Duplicate field 1 in PF4Enum".to_string());
                } else {
                    type_name = Some(result?);
                }
            } else if let Some(result) = self.nested_string(field, 2) {
                if value.is_some() {
                    self.warnings.push("Duplicate field 2 in PF4Enum".to_string());
                } else {
                    value = Some(result?);
                }
            } else {
                self.warnings.push(format!("Unexpected field {:?} in PF4Enum", field));
            }
        }
        if let (Some(type_name), Some(value)) = (type_name, value) {
            Ok(PF4Enum { type_name, value })
        } else {
            Err("Missing required fields in PF4Enum".to_string())
        }
    }

    fn parse_nlg_data(&mut self, bytes: &[u8]) -> Result<PF4NLGData, String> {
        let mut id = None;
        let mut text = None;
        let fields = parse_fields(bytes)?;
        for field in &fields {
            if let Some(result) = self.nested_string(field, 1) {
                if id.is_some() {
                    self.warnings.push("Duplicate field 1 in PF4NLGData".to_string());
                } else {
                    id = Some(result?);
                }
            } else if let Some(result) = self.nested_string(field, 2) {
                if text.is_some() {
                    self.warnings.push("Duplicate field 2 in PF4NLGData".to_string());
                } else {
                    text = Some(result?);
                }
            } else {
                self.warnings.push(format!("Unexpected field {:?} in PF4NLGData", field));
            }
        }
        if let (Some(id), Some(text)) = (id, text) {
            Ok(PF4NLGData { id, text })
        } else {
            Err("Missing required fields in PF4NLGData".to_string())
        }
    }

    fn parse_key_value_array(&mut self, bytes: &[u8]) -> Result<Vec<PF4KeyValue>, String> {
        let mut key_values = Vec::new();
        let fields = parse_fields(bytes)?;
        for field in &fields {
            if let Some(result) = self.nested_object(field, 1, Self::parse_key_value) {
                key_values.push(result?);
            } else {
                self.warnings.push(format!("Unexpected field {:?} in PF4KeyValue array", field));
            }
        }
        Ok(key_values)
    }

    fn parse_command(&mut self, bytes: &[u8]) -> Result<PF4Command, String> {
        let mut value = None;
        let fields = parse_fields(bytes)?;
        for field in &fields {
            if let Some(result) = self.nested_object(field, 3, Self::parse_key_value) {
                if value.is_some() {
                    self.warnings.push("Duplicate field 3 in PF4Command".to_string());
                } else {
                    value = Some(result?);
                }
            } else {
                self.warnings.push(format!("Unexpected field {:?} in PF4Command", field));
            }
        }
        if let Some(value) = value {
            Ok(PF4Command { value })
        } else {
            Err("Missing required field 3 in PF4Command".to_string())
        }
    }
}

fn jikkenf(s64: &str) {
    println!("jikken: {}", s64);
    let s = BASE64_STANDARD.decode(s64).expect("failed to decode base64");
    let dump = dump_proto(&s, 0).expect("failed to dump proto");
    println!("{}", dump);
    let mut parser = Parser::new();
    println!("{:?}", parser.parse(&s));
    println!("warnings: {:?}", parser.warnings);
}

pub fn jikken() {
    jikkenf("EpQCEuYBChMKCGRpc3RhbmNlMQAAAAAAQH9ACjcKDWRpc3RhbmNlX3VuaXRaJgoXbmxwX2dlbmVyYXRpb24uVW5pdFR5cGUSC1VOSVRfTUVURVJTCpUBCghtYW5ldXZlcoIBhwEKIwoDa2V5chwaGgoLcGF0aGZpbmRlcjQSC3BmX3R1cm5zdGVwCmAKBGFyZ3OCAVcKJwoSbGFuZV9ndWlkYW5jZV90eXBlIhFVU0VfUklHSFRfMl9MQU5FUwoYCg50dXJuX3NoYXJwbmVzcyIGTk9STUFMChIKCXR1cm5fc2lkZSIFUklHSFQqKRonCgtwYXRoZmluZGVyNBIYcHJlcGFyZV9kaXN0YW5jZV9tZXNzYWdl");
    jikkenf("EukBErsBChMKCGRpc3RhbmNlMQAAAAAAAHlACjcKDWRpc3RhbmNlX3VuaXRaJgoXbmxwX2dlbmVyYXRpb24uVW5pdFR5cGUSC1VOSVRfTUVURVJTCmsKCG1hbmV1dmVyggFeCiMKA2tleXIcGhoKC3BhdGhmaW5kZXI0EgtwZl90dXJuc3RlcAo3CgRhcmdzggEuChgKDnR1cm5fc2hhcnBuZXNzIgZOT1JNQUwKEgoJdHVybl9zaWRlIgVSSUdIVCopGicKC3BhdGhmaW5kZXI0EhhwcmVwYXJlX2Rpc3RhbmNlX21lc3NhZ2U=");
    jikkenf("EqYCEvgBChMKCGRpc3RhbmNlMQAAAAAAAHlACjcKDWRpc3RhbmNlX3VuaXRaJgoXbmxwX2dlbmVyYXRpb24uVW5pdFR5cGUSC1VOSVRfTUVURVJTCqcBCghtYW5ldXZlcoIBmQEKJgoDa2V5ch8aHQoLcGF0aGZpbmRlcjQSDnBmX29mZnJhbXBzdGVwCm8KBGFyZ3OCAWYKZAoJZXhpdF9uYW1lggFWClQKBWV4aXRzggFKCkh6RgoeL0ZFQVRVUkVfSUQvMHhmMmQ1ZDAxYjg0YTJjMzUyEiTkuqzokYnluILlt53jgqTjg7Pjgr/jg7zjg4Hjgqfjg7PjgrgqKRonCgtwYXRoZmluZGVyNBIYcHJlcGFyZV9kaXN0YW5jZV9tZXNzYWdl");
    jikkenf("EqYDEvICCqkBCgpmaXJzdF9zdGVwggGZAQomCgNrZXlyHxodCgtwYXRoZmluZGVyNBIOcGZfb2ZmcmFtcHN0ZXAKbwoEYXJnc4IBZgpkCglleGl0X25hbWWCAVYKVAoFZXhpdHOCAUoKSHpGCh4vRkVBVFVSRV9JRC8weGYyZDVkMDFiODRhMmMzNTISJOS6rOiRieW4guW3neOCpOODs+OCv+ODvOODgeOCp+ODs+OCuArDAQoLc2Vjb25kX3N0ZXCCAbIBCiUKA2tleXIeGhwKC3BhdGhmaW5kZXI0Eg1wZl9vbnJhbXBzdGVwCogBCgRhcmdzggF/ChgKDnR1cm5fc2hhcnBuZXNzIgZTTElHSFQKEQoJdHVybl9zaWRlIgRMRUZUClAKEnNpZ25faW5kaXJlY3RfbmFtZYIBOQo3CgZyb3V0ZXOCASwKKnooCh4vRkVBVFVSRV9JRC8weGZmMjBlOTlhZjkxYWM1MzkSBuW4guW3nSovGi0KC3BhdGhmaW5kZXI0Eh5jb21iaW5lX21lcmdlZF9ndWlkYW5jZV9ldmVudHM=");
    jikkenf("EugCErQCCsIBCgpmaXJzdF9zdGVwggGyAQolCgNrZXlyHhocCgtwYXRoZmluZGVyNBINcGZfb25yYW1wc3RlcAqIAQoEYXJnc4IBfwoYCg50dXJuX3NoYXJwbmVzcyIGU0xJR0hUChEKCXR1cm5fc2lkZSIETEVGVApQChJzaWduX2luZGlyZWN0X25hbWWCATkKNwoGcm91dGVzggEsCip6KAoeL0ZFQVRVUkVfSUQvMHhmZjIwZTk5YWY5MWFjNTM5EgbluILlt50KbQoLc2Vjb25kX3N0ZXCCAV0KIwoDa2V5chwaGgoLcGF0aGZpbmRlcjQSC3BmX3R1cm5zdGVwCjYKBGFyZ3OCAS0KGAoOdHVybl9zaGFycG5lc3MiBk5PUk1BTAoRCgl0dXJuX3NpZGUiBExFRlQqLxotCgtwYXRoZmluZGVyNBIeY29tYmluZV9tZXJnZWRfZ3VpZGFuY2VfZXZlbnRz");
    jikkenf("EmASQAoRCg10cmFmZmljX2xpZ2h0GAEKGAoOdHVybl9zaGFycG5lc3MiBk5PUk1BTAoRCgl0dXJuX3NpZGUiBExFRlQqHBoaCgtwYXRoZmluZGVyNBILcGZfdHVybnN0ZXA=");
    jikkenf("EnwSUgoTCghkaXN0YW5jZTEAAAAAAADwPwo7Cg1kaXN0YW5jZV91bml0WioKF25scF9nZW5lcmF0aW9uLlVuaXRUeXBlEg9VTklUX0tJTE9NRVRFUlMqJhokCgtwYXRoZmluZGVyNBIVY29udGludWVfZm9yX2Rpc3RhbmNl");
    jikkenf("EukBErsBChMKCGRpc3RhbmNlMQAAAAAAwHJACjcKDWRpc3RhbmNlX3VuaXRaJgoXbmxwX2dlbmVyYXRpb24uVW5pdFR5cGUSC1VOSVRfTUVURVJTCmsKCG1hbmV1dmVyggFeCiMKA2tleXIcGhoKC3BhdGhmaW5kZXI0EgtwZl90dXJuc3RlcAo3CgRhcmdzggEuChgKDnR1cm5fc2hhcnBuZXNzIgZOT1JNQUwKEgoJdHVybl9zaWRlIgVSSUdIVCopGicKC3BhdGhmaW5kZXI0EhhwcmVwYXJlX2Rpc3RhbmNlX21lc3NhZ2U=");
    jikkenf("Ek4SLgoYCg50dXJuX3NoYXJwbmVzcyIGTk9STUFMChIKCXR1cm5fc2lkZSIFUklHSFQqHBoaCgtwYXRoZmluZGVyNBILcGZfdHVybnN0ZXA=");
    jikkenf("EsoBEpwBChMKCGRpc3RhbmNlMQAAAAAAAGlACjcKDWRpc3RhbmNlX3VuaXRaJgoXbmxwX2dlbmVyYXRpb24uVW5pdFR5cGUSC1VOSVRfTUVURVJTCkwKCG1hbmV1dmVyggE/CjIKA2tleXIrGikKC3BhdGhmaW5kZXI0EhpwZl9kZXN0aW5hdGlvbnN0ZXBfcHJlcGFyZQoJCgRhcmdzggEAKikaJwoLcGF0aGZpbmRlcjQSGHByZXBhcmVfZGlzdGFuY2VfbWVzc2FnZQ==");
    jikkenf("EisSAConGiUKC3BhdGhmaW5kZXI0EhZwZl9kZXN0aW5hdGlvbnN0ZXBfYWN0");
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CASES: &[&str] = &[
        "EpQCEuYBChMKCGRpc3RhbmNlMQAAAAAAQH9ACjcKDWRpc3RhbmNlX3VuaXRaJgoXbmxwX2dlbmVyYXRpb24uVW5pdFR5cGUSC1VOSVRfTUVURVJTCpUBCghtYW5ldXZlcoIBhwEKIwoDa2V5chwaGgoLcGF0aGZpbmRlcjQSC3BmX3R1cm5zdGVwCmAKBGFyZ3OCAVcKJwoSbGFuZV9ndWlkYW5jZV90eXBlIhFVU0VfUklHSFRfMl9MQU5FUwoYCg50dXJuX3NoYXJwbmVzcyIGTk9STUFMChIKCXR1cm5fc2lkZSIFUklHSFQqKRonCgtwYXRoZmluZGVyNBIYcHJlcGFyZV9kaXN0YW5jZV9tZXNzYWdl",
        "EukBErsBChMKCGRpc3RhbmNlMQAAAAAAAHlACjcKDWRpc3RhbmNlX3VuaXRaJgoXbmxwX2dlbmVyYXRpb24uVW5pdFR5cGUSC1VOSVRfTUVURVJTCmsKCG1hbmV1dmVyggFeCiMKA2tleXIcGhoKC3BhdGhmaW5kZXI0EgtwZl90dXJuc3RlcAo3CgRhcmdzggEuChgKDnR1cm5fc2hhcnBuZXNzIgZOT1JNQUwKEgoJdHVybl9zaWRlIgVSSUdIVCopGicKC3BhdGhmaW5kZXI0EhhwcmVwYXJlX2Rpc3RhbmNlX21lc3NhZ2U=",
        "EqYCEvgBChMKCGRpc3RhbmNlMQAAAAAAAHlACjcKDWRpc3RhbmNlX3VuaXRaJgoXbmxwX2dlbmVyYXRpb24uVW5pdFR5cGUSC1VOSVRfTUVURVJTCqcBCghtYW5ldXZlcoIBmQEKJgoDa2V5ch8aHQoLcGF0aGZpbmRlcjQSDnBmX29mZnJhbXBzdGVwCm8KBGFyZ3OCAWYKZAoJZXhpdF9uYW1lggFWClQKBWV4aXRzggFKCkh6RgoeL0ZFQVRVUkVfSUQvMHhmMmQ1ZDAxYjg0YTJjMzUyEiTkuqzokYnluILlt53jgqTjg7Pjgr/jg7zjg4Hjgqfjg7PjgrgqKRonCgtwYXRoZmluZGVyNBIYcHJlcGFyZV9kaXN0YW5jZV9tZXNzYWdl",
        "EqYDEvICCqkBCgpmaXJzdF9zdGVwggGZAQomCgNrZXlyHxodCgtwYXRoZmluZGVyNBIOcGZfb2ZmcmFtcHN0ZXAKbwoEYXJnc4IBZgpkCglleGl0X25hbWWCAVYKVAoFZXhpdHOCAUoKSHpGCh4vRkVBVFVSRV9JRC8weGYyZDVkMDFiODRhMmMzNTISJOS6rOiRieW4guW3neOCpOODs+OCv+ODvOODgeOCp+ODs+OCuArDAQoLc2Vjb25kX3N0ZXCCAbIBCiUKA2tleXIeGhwKC3BhdGhmaW5kZXI0Eg1wZl9vbnJhbXBzdGVwCogBCgRhcmdzggF/ChgKDnR1cm5fc2hhcnBuZXNzIgZTTElHSFQKEQoJdHVybl9zaWRlIgRMRUZUClAKEnNpZ25faW5kaXJlY3RfbmFtZYIBOQo3CgZyb3V0ZXOCASwKKnooCh4vRkVBVFVSRV9JRC8weGZmMjBlOTlhZjkxYWM1MzkSBuW4guW3nSovGi0KC3BhdGhmaW5kZXI0Eh5jb21iaW5lX21lcmdlZF9ndWlkYW5jZV9ldmVudHM=",
        "EugCErQCCsIBCgpmaXJzdF9zdGVwggGyAQolCgNrZXlyHhocCgtwYXRoZmluZGVyNBINcGZfb25yYW1wc3RlcAqIAQoEYXJnc4IBfwoYCg50dXJuX3NoYXJwbmVzcyIGU0xJR0hUChEKCXR1cm5fc2lkZSIETEVGVApQChJzaWduX2luZGlyZWN0X25hbWWCATkKNwoGcm91dGVzggEsCip6KAoeL0ZFQVRVUkVfSUQvMHhmZjIwZTk5YWY5MWFjNTM5EgbluILlt50KbQoLc2Vjb25kX3N0ZXCCAV0KIwoDa2V5chwaGgoLcGF0aGZpbmRlcjQSC3BmX3R1cm5zdGVwCjYKBGFyZ3OCAS0KGAoOdHVybl9zaGFycG5lc3MiBk5PUk1BTAoRCgl0dXJuX3NpZGUiBExFRlQqLxotCgtwYXRoZmluZGVyNBIeY29tYmluZV9tZXJnZWRfZ3VpZGFuY2VfZXZlbnRz",
        "EmASQAoRCg10cmFmZmljX2xpZ2h0GAEKGAoOdHVybl9zaGFycG5lc3MiBk5PUk1BTAoRCgl0dXJuX3NpZGUiBExFRlQqHBoaCgtwYXRoZmluZGVyNBILcGZfdHVybnN0ZXA=",
        "EnwSUgoTCghkaXN0YW5jZTEAAAAAAADwPwo7Cg1kaXN0YW5jZV91bml0WioKF25scF9nZW5lcmF0aW9uLlVuaXRUeXBlEg9VTklUX0tJTE9NRVRFUlMqJhokCgtwYXRoZmluZGVyNBIVY29udGludWVfZm9yX2Rpc3RhbmNl",
        "EukBErsBChMKCGRpc3RhbmNlMQAAAAAAwHJACjcKDWRpc3RhbmNlX3VuaXRaJgoXbmxwX2dlbmVyYXRpb24uVW5pdFR5cGUSC1VOSVRfTUVURVJTCmsKCG1hbmV1dmVyggFeCiMKA2tleXIcGhoKC3BhdGhmaW5kZXI0EgtwZl90dXJuc3RlcAo3CgRhcmdzggEuChgKDnR1cm5fc2hhcnBuZXNzIgZOT1JNQUwKEgoJdHVybl9zaWRlIgVSSUdIVCopGicKC3BhdGhmaW5kZXI0EhhwcmVwYXJlX2Rpc3RhbmNlX21lc3NhZ2U=",
        "Ek4SLgoYCg50dXJuX3NoYXJwbmVzcyIGTk9STUFMChIKCXR1cm5fc2lkZSIFUklHSFQqHBoaCgtwYXRoZmluZGVyNBILcGZfdHVybnN0ZXA=",
        "EsoBEpwBChMKCGRpc3RhbmNlMQAAAAAAAGlACjcKDWRpc3RhbmNlX3VuaXRaJgoXbmxwX2dlbmVyYXRpb24uVW5pdFR5cGUSC1VOSVRfTUVURVJTCkwKCG1hbmV1dmVyggE/CjIKA2tleXIrGikKC3BhdGhmaW5kZXI0EhpwZl9kZXN0aW5hdGlvbnN0ZXBfcHJlcGFyZQoJCgRhcmdzggEAKikaJwoLcGF0aGZpbmRlcjQSGHByZXBhcmVfZGlzdGFuY2VfbWVzc2FnZQ==",
        "EisSAConGiUKC3BhdGhmaW5kZXI0EhZwZl9kZXN0aW5hdGlvbnN0ZXBfYWN0",
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
    fn test_jikken_warnings() {
        for s in TEST_CASES {
            parse_and_warn(s);
        }
    }
}

