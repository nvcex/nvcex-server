use axum::{http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use base64::prelude::*;
use protobuf_core::{Field, FieldNumber, FieldValue, IteratorExtProtobuf, AsRefExtProtobuf};
mod pathfinder4;

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


#[tokio::main]
async fn main() {
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
    let s = BASE64_STANDARD.decode(&payload.body).expect("failed to decode base64");
    let mut parser = pathfinder4::Parser::new();
    let message = parser.parse(&s).unwrap();
    println!("Parsed message: {:?}", message);
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
