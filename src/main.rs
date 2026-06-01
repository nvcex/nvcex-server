use axum::{http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use base64::prelude::*;
use protobuf_core::{Field, FieldNumber, FieldValue, IteratorExtProtobuf, AsRefExtProtobuf};

#[derive(Debug, Deserialize)]
struct TtsRequest {
    text: String,
    voice_type: String,
    format: Option<String>,
}

#[derive(Debug, Serialize)]
struct TtsResponse {
    status: String,
    voice_id: String,
    format: String,
    source_url: String,
}

#[derive(Debug, Deserialize)]
struct LogTextRequest {
    text: String,
    body: String  // base64-encoded
}

fn parse_fields(bytes: &[u8]) -> Vec<Field<&[u8]>> {
    AsRefExtProtobuf::read_protobuf_fields(bytes)
        .collect::<Result<Vec<Field<&[u8]>>, _>>()
        .unwrap()
}

fn level_3_2_1_16_1_16_1(bytes: &[u8]) {
    println!("              ( // 3-2-1-16-1-16-1");
    let fields = parse_fields(bytes);
    for field in &fields {
        if field.field_number.as_u32() == 1 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    let s = str::from_utf8(nested_bytes).unwrap_or("<invalid utf-8>");
                    println!("        decoded string: {}", s);
                }
                _ => {
                    println!("unexpected field type for field 1: {:?}", field.value);
                }
            }
        } else if field.field_number.as_u32() == 4 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    let s = str::from_utf8(nested_bytes).unwrap_or("<invalid utf-8>");
                    println!("        decoded string: {}", s);
                }
                _ => {
                    println!("unexpected field type for field 4: {:?}", field.value);
                }
            }
        } else {
            println!("        unknown field: {:?}", field);
        }
    }
    println!("              )");
}

fn level_3_2_1_16_1_14_3(bytes: &[u8]) {
    println!("            ( // 3-2-1-16-1-14-3");
    let fields = parse_fields(bytes);
    for field in &fields {
        if field.field_number.as_u32() == 1 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    let s = str::from_utf8(nested_bytes).unwrap_or("<invalid utf-8>");
                    println!("        decoded string: {}", s);
                }
                _ => {
                    println!("unexpected field type for field 1: {:?}", field.value);
                }
            }
        } else if field.field_number.as_u32() == 2 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    let s = str::from_utf8(nested_bytes).unwrap_or("<invalid utf-8>");
                    println!("        decoded string: {}", s);
                }
                _ => {
                    println!("unexpected field type for field 2: {:?}", field.value);
                }
            }
        } else {
            println!("        unknown field: {:?}", field);
        }
    }
    println!("            )");
}

fn level_3_2_1_16_1_14(bytes: &[u8]) {
    println!("            ( // 3-2-1-16-1-14");
    let fields = parse_fields(bytes);
    for field in &fields {
        if field.field_number.as_u32() == 3 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    level_3_2_1_16_1_14_3(nested_bytes);
                }
                _ => {
                    println!("unexpected field type for field 3: {:?}", field.value);
                }
            }
        } else {
            println!("            unknown field: {:?}", field);
        }
    }
    println!("            )");
}

fn level_3_2_1_16_1_16(bytes: &[u8]) {
    println!("            ( // 3-2-1-16-1-16");
    let fields = parse_fields(bytes);
    for field in &fields {
        if field.field_number.as_u32() == 1 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    level_3_2_1_16_1_16_1(nested_bytes);
                }
                _ => {
                    println!("unexpected field type for field 1: {:?}", field.value);
                }
            }
        } else {
            println!("            unknown field: {:?}", field);
        }
    }
    println!("            )");
}

fn level_3_2_1_16_1(bytes: &[u8]) {
    println!("          ( // 3-2-1-16-1");
    let fields = parse_fields(bytes);
    for field in &fields {
        if field.field_number.as_u32() == 1 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    let s = str::from_utf8(nested_bytes).unwrap_or("<invalid utf-8>");
                    println!("          decoded string: {}", s);
                }
                _ => {
                    println!("unexpected field type for field 1: {:?}", field.value);
                }
            }
        } else if field.field_number.as_u32() == 14 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    level_3_2_1_16_1_14(nested_bytes);
                }
                _ => {
                    println!("unexpected field type for field 14: {:?}", field.value);
                }
            }
        } else if field.field_number.as_u32() == 16 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    level_3_2_1_16_1_16(nested_bytes);
                }
                _ => {
                    println!("unexpected field type for field 16: {:?}", field.value);
                }
            }
        } else {
            println!("          unknown field: {:?}", field);
        }
    }
    println!("          )");
}

fn level_3_2_1_11(bytes: &[u8]) {
    println!("        ( // 3-2-1-11");
    let fields = parse_fields(bytes);
    for field in &fields {
        if field.field_number.as_u32() == 1 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    let s = str::from_utf8(nested_bytes).unwrap_or("<invalid utf-8>");
                    println!("        decoded string: {}", s);
                }
                _ => {
                    println!("unexpected field type for field 1: {:?}", field.value);
                }
            }
        } else if field.field_number.as_u32() == 2 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    let s = str::from_utf8(nested_bytes).unwrap_or("<invalid utf-8>");
                    println!("        decoded string: {}", s);
                }
                _ => {
                    println!("unexpected field type for field 2: {:?}", field.value);
                }
            }
        } else {
            println!("        unknown field: {:?}", field);
        }
    }
    println!("        )");
}

fn level_3_2_1_16(bytes: &[u8]) {
    println!("        ( // 3-2-1-16");
    let fields = parse_fields(bytes);
    for field in &fields {
        if field.field_number.as_u32() == 1 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    level_3_2_1_16_1(nested_bytes);
                }
                _ => {
                    println!("unexpected field type for field 1: {:?}", field.value);
                }
            }
        } else {
            println!("        unknown field: {:?}", field);
        }
    }
    println!("        )");
}

fn level_3_2_1(bytes: &[u8]) {
    println!("      ( // 3-2-1");
    let fields = parse_fields(bytes);
    for field in &fields {
        if field.field_number.as_u32() == 1 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    let s = str::from_utf8(nested_bytes).unwrap_or("<invalid utf-8>");
                    println!("      decoded string: {}", s);
                }
                _ => {
                    println!("unexpected field type for field 1: {:?}", field.value);
                }
            }
        } else if field.field_number.as_u32() == 6 {
            match field.value {
                FieldValue::I64(value) => {
                    let v = f64::from_le_bytes(value);
                    println!("      decoded f64: {}", v);
                }
                _ => {
                    println!("unexpected field type for field 6: {:?}", field.value);
                }
            }
        } else if field.field_number.as_u32() == 11 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    level_3_2_1_11(nested_bytes);
                }
                _ => {
                    println!("unexpected field type for field 11: {:?}", field.value);
                }
            }
        } else if field.field_number.as_u32() == 16 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    level_3_2_1_16(nested_bytes);
                }
                _ => {
                    println!("unexpected field type for field 16: {:?}", field.value);
                }
            }
        } else {
            println!("      unknown field: {:?}", field);
        }
    }
    println!("      )");
}

fn level_3_5_3(bytes: &[u8]) {
    println!("      ( // 3-5-3");
    let fields = parse_fields(bytes);
    for field in &fields {
        if field.field_number.as_u32() == 1 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    let s = str::from_utf8(nested_bytes).unwrap_or("<invalid utf-8>");
                    println!("        decoded string: {}", s);
                }
                _ => {
                    println!("unexpected field type for field 1: {:?}", field.value);
                }
            }
        } else if field.field_number.as_u32() == 2 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    let s = str::from_utf8(nested_bytes).unwrap_or("<invalid utf-8>");
                    println!("        decoded string: {}", s);
                }
                _ => {
                    println!("unexpected field type for field 2: {:?}", field.value);
                }
            }
        } else {
            println!("        unknown field: {:?}", field);
        }
    }
    println!("      )");
}

fn level_3_2(bytes: &[u8]) {
    println!("    ( // 3-2");
    let fields = parse_fields(bytes);
    for field in &fields {
        if field.field_number.as_u32() == 1 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    level_3_2_1(nested_bytes);
                }
                _ => {
                    println!("unexpected field type for field 1: {:?}", field.value);
                }
            }
        } else {
            println!("    unknown field: {:?}", field);
        }
    }
    println!("    )");
}

fn level_3_5(bytes: &[u8]) {
    println!("    ( // 3-5");
    let fields = parse_fields(bytes);
    for field in &fields {
        if field.field_number.as_u32() == 3 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    level_3_5_3(nested_bytes);
                }
                _ => {
                    println!("unexpected field type for field 3: {:?}", field.value);
                }
            }
        } else {
            println!("    unknown field: {:?}", field);
        }
    }
    println!("    )");
}

fn level_2(bytes: &[u8]) {
    println!("  ( // 2");
    let fields = parse_fields(bytes);
    for field in &fields {
        if field.field_number.as_u32() == 2 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    level_3_2(nested_bytes);
                }
                _ => {
                    println!("unexpected field type for field 2: {:?}", field.value);
                }
            }
        } else if field.field_number.as_u32() == 5 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    level_3_5(nested_bytes);
                }
                _ => {
                    println!("unexpected field type for field 5: {:?}", field.value);
                }
            }
        } else {
            println!("  unknown field: {:?}", field);
        }
    }
    println!("  )");
}

fn level_1(bytes: &[u8]) {
    println!("( // 1");
    let fields = parse_fields(bytes);
    for field in &fields {
        if field.field_number.as_u32() == 2 {
            match field.value {
                FieldValue::Len(nested_bytes) => {
                    level_2(nested_bytes);
                }
                _ => {
                    println!("unexpected field type for field 2: {:?}", field.value);
                }
            }
        } else {
            println!("unknown field: {:?}", field);
        }
    }
    println!(")");
    println!();
}

fn dump_proto(bytes: &[u8], indent: usize) {
    println!("{}(", " ".repeat(indent));
    let fields = parse_fields(bytes);
    for field in &fields {
        match &field.value {
            FieldValue::Len(nested_bytes) => {
                let s = str::from_utf8(nested_bytes);
                match s {
                    Ok(s) if s.chars().all(|c| !c.is_control()) => {
                        println!("{}{}: {}", " ".repeat(indent), field.field_number.as_u32(), s);
                    }
                    _ => {
                        println!("{}{}: (binary data, length {})", " ".repeat(indent), field.field_number.as_u32(), nested_bytes.len());
                        dump_proto(nested_bytes, indent + 2);
                    }
                }
            }
            other => println!("{}{}: {:?}", " ".repeat(indent), field.field_number.as_u32(), other),
        }
    }
    println!("{})", " ".repeat(indent));
}

#[derive(Debug)]
struct StructuredMessage {
    // field 2: PF4Message
    body: PF4Message
}

#[derive(Debug)]
struct PF4Message {
    // field 2: PF4MessageFields
    fields: PF4MessageFields,
    // field 5: PF4Command
    command: PF4Command
}

type PF4MessageFields = Vec<PF4KeyValue>;

#[derive(Debug)]
struct PF4Command {
    // field 3: PF4KeyValue
    value: PF4KeyValue
}

#[derive(Debug)]
enum PF4Value {
    // field 2: string
    StringValue(String),
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
    Unknown
}

#[derive(Debug)]
struct PF4Enum {
    // field 1: string
    type_name: String,
    // field 2: string
    value: String
}

#[derive(Debug)]
struct PF4NLGData {
    // field 1: string
    id: String,
    // field 2: string
    text: String
}

#[derive(Debug)]
struct PF4KeyValue {
    // field 1: string
    key: String,
    value: PF4Value
}

struct Parser {
    warnings: Vec<String>
}

impl Parser {
    fn new() -> Self {
        Parser { warnings: Vec::new() }
    }

    fn parse(&mut self, bytes: &[u8]) -> Result<StructuredMessage, String> {
        let mut body = None;
        let fields = parse_fields(bytes);
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
                return Some(bytes_to_utf8_string(nested_bytes));
            }
        }
        None
    }

    fn parse_pf4_message(&mut self, bytes: &[u8]) -> Result<PF4Message, String> {
        let mut message_fields = None;
        let mut command = None;
        let fields = parse_fields(bytes);
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
        let fields = parse_fields(bytes);
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
        let fields = parse_fields(bytes);
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
        let fields = parse_fields(bytes);
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
        let fields = parse_fields(bytes);
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
        let fields = parse_fields(bytes);
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
        let fields = parse_fields(bytes);
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

fn jikken(s64: &str) {
    println!("jikken: {}", s64);
    let s = BASE64_STANDARD.decode(s64).expect("failed to decode base64");
    level_1(&s);
    dump_proto(&s, 0);
    let mut parser = Parser::new();
    println!("{:?}", parser.parse(&s));
    println!("warnings: {:?}", parser.warnings);
}

#[tokio::main]
async fn main() {
    jikken("EpQCEuYBChMKCGRpc3RhbmNlMQAAAAAAQH9ACjcKDWRpc3RhbmNlX3VuaXRaJgoXbmxwX2dlbmVyYXRpb24uVW5pdFR5cGUSC1VOSVRfTUVURVJTCpUBCghtYW5ldXZlcoIBhwEKIwoDa2V5chwaGgoLcGF0aGZpbmRlcjQSC3BmX3R1cm5zdGVwCmAKBGFyZ3OCAVcKJwoSbGFuZV9ndWlkYW5jZV90eXBlIhFVU0VfUklHSFRfMl9MQU5FUwoYCg50dXJuX3NoYXJwbmVzcyIGTk9STUFMChIKCXR1cm5fc2lkZSIFUklHSFQqKRonCgtwYXRoZmluZGVyNBIYcHJlcGFyZV9kaXN0YW5jZV9tZXNzYWdl");
    jikken("EukBErsBChMKCGRpc3RhbmNlMQAAAAAAAHlACjcKDWRpc3RhbmNlX3VuaXRaJgoXbmxwX2dlbmVyYXRpb24uVW5pdFR5cGUSC1VOSVRfTUVURVJTCmsKCG1hbmV1dmVyggFeCiMKA2tleXIcGhoKC3BhdGhmaW5kZXI0EgtwZl90dXJuc3RlcAo3CgRhcmdzggEuChgKDnR1cm5fc2hhcnBuZXNzIgZOT1JNQUwKEgoJdHVybl9zaWRlIgVSSUdIVCopGicKC3BhdGhmaW5kZXI0EhhwcmVwYXJlX2Rpc3RhbmNlX21lc3NhZ2U=");
    jikken("EqYCEvgBChMKCGRpc3RhbmNlMQAAAAAAAHlACjcKDWRpc3RhbmNlX3VuaXRaJgoXbmxwX2dlbmVyYXRpb24uVW5pdFR5cGUSC1VOSVRfTUVURVJTCqcBCghtYW5ldXZlcoIBmQEKJgoDa2V5ch8aHQoLcGF0aGZpbmRlcjQSDnBmX29mZnJhbXBzdGVwCm8KBGFyZ3OCAWYKZAoJZXhpdF9uYW1lggFWClQKBWV4aXRzggFKCkh6RgoeL0ZFQVRVUkVfSUQvMHhmMmQ1ZDAxYjg0YTJjMzUyEiTkuqzokYnluILlt53jgqTjg7Pjgr/jg7zjg4Hjgqfjg7PjgrgqKRonCgtwYXRoZmluZGVyNBIYcHJlcGFyZV9kaXN0YW5jZV9tZXNzYWdl");
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/tts", post(handle_tts_request))
        .route("/log_text", post(handle_log_text));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing_subscriber::fmt().init();
    println!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind to address");

    axum::serve(listener, app.into_make_service())
        .await
        .expect("server failed");
}

async fn hello_world() -> impl IntoResponse {
    (StatusCode::OK, "Hello, world!")
}

async fn handle_log_text(Json(payload): Json<LogTextRequest>) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!(?payload, "received log_text request");
    Ok(Json(serde_json::json!({"status": "success"})))
}

async fn handle_tts_request(Json(payload): Json<TtsRequest>) -> Result<Json<TtsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let format = payload
        .format
        .as_deref()
        .unwrap_or("mp3")
        .to_lowercase();

    if payload.text.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json_error("text must not be empty"))));
    }

    let voice_id = match map_voice_type(&payload.voice_type) {
        Some(id) => id,
        None => {
            return Err(
                (
                    StatusCode::BAD_REQUEST,
                    Json(json_error("unsupported voice_type")),
                ),
            )
        }
    };

    if !matches!(format.as_str(), "mp3" | "ogg") {
        return Err(
            (
                StatusCode::BAD_REQUEST,
                Json(json_error("format must be mp3 or ogg")),
            ),
        );
    }

    let client = Client::new();
    let tts_backend_url = "https://example-tts-backend.local/generate";

    let mut request_body = HashMap::new();
    request_body.insert("text", payload.text);
    request_body.insert("voice_id", voice_id.clone());
    request_body.insert("format", format.clone());

    let backend_response = client
        .post(tts_backend_url)
        .json(&request_body)
        .send()
        .await;

    let backend_response = match backend_response {
        Ok(res) => res,
        Err(err) => {
            tracing::error!(?err, "failed to call TTS backend");
            return Err(
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json_error("failed to communicate with TTS backend")),
                ),
            );
        }
    };

    if !backend_response.status().is_success() {
        tracing::error!(status = ?backend_response.status(), "tts backend returned error");
        return Err(
            (
                StatusCode::BAD_GATEWAY,
                Json(json_error("TTS backend returned an error")),
            ),
        );
    }

    let backend_payload: BackendResponse = match backend_response.json().await {
        Ok(body) => body,
        Err(err) => {
            tracing::error!(?err, "failed to parse TTS backend response");
            return Err(
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json_error("invalid response from TTS backend")),
                ),
            );
        }
    };

    let response = TtsResponse {
        status: "ok".to_string(),
        voice_id,
        format,
        source_url: backend_payload.source_url,
    };

    Ok(Json(response))
}

fn map_voice_type(voice_type: &str) -> Option<String> {
    let voices = HashMap::from([
        ("standard", "voice_standard"),
        ("soft", "voice_soft"),
        ("bright", "voice_bright"),
    ]);

    voices.get(voice_type).map(|s| s.to_string())
}

fn json_error(message: &str) -> serde_json::Value {
    serde_json::json!({ "error": message })
}

fn bytes_to_utf8_string(bytes: &[u8]) -> Result<String, String> {
    match std::str::from_utf8(bytes) {
        Ok(s) => Ok(s.to_string()),
        Err(e) => Err(format!("invalid utf-8 sequence: {}", e)),
    }
}

#[derive(Debug, Deserialize)]
struct BackendResponse {
    source_url: String,
}
