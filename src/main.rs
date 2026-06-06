use axum::{http::StatusCode, response::{Html, IntoResponse}, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use base64::prelude::*;
mod pathfinder4;
mod scenarios;
mod voices;
use crate::voices::{SpeechText, Speaker, Voice, VoicevoxClient};
use crate::scenarios::{build_voicevox_scenarios, SharedScenario};
use axum::extract::State;

#[derive(Clone)]
struct AppState {
    voicevox: VoicevoxClient,
    scenarios: HashMap<String, SharedScenario>,
}

#[derive(Debug, Deserialize)]
struct TtsRequest {
    text: String,
    data: String, // base64-encoded
    voice_type: String,
    format: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct LogTextRequest {
    text: String,
    data: String, // base64-encoded
}

#[tokio::main]
async fn main() {
    let voicevox = VoicevoxClient::new("http://localhost:50000");
    let supported_speakers = match voicevox.speakers().await {
        Ok(list) => list,
        Err(err) => {
            tracing::warn!(?err, "failed to fetch voicevox speakers, continuing with empty scenarios");
            Vec::new()
        }
    };
    let scenarios = build_voicevox_scenarios(&supported_speakers);

    let state = Arc::new(AppState {
        voicevox,
        scenarios,
    });

    let app = Router::new()
        .route("/", get(index_html))
        .route("/test", get(test_html))
        .route("/sample.json", get(sample_json))
        .route("/parse", post(handle_parse_request))
        .route("/tts", post(handle_tts_request))
        .route("/log_text", post(handle_log_text))
        .route("/voicevox/speakers", get(handle_voicevox_speakers))
        .route("/scenarios", get(handle_scenarios))
        .with_state(state.clone());

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

const INDEX_HTML: &str = include_str!("../static/index.html");
const TEST_HTML: &str = include_str!("../static/test.html");

async fn index_html() -> impl IntoResponse {
    Html(INDEX_HTML)
}

async fn test_html() -> impl IntoResponse {
    Html(TEST_HTML)
}

const SAMPLE_JSON: &str = include_str!("../static/sample.json");

async fn sample_json() -> impl IntoResponse {
    Json(
        serde_json::from_str::<serde_json::Value>(SAMPLE_JSON)
            .expect("embedded sample.json must be valid JSON"),
    )
}

async fn handle_parse_request(Json(payload): Json<TtsRequest>) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if payload.data.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json_error("data must not be empty"))));
    }

    let body_bytes = match BASE64_STANDARD.decode(&payload.data) {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!(?err, "failed to decode base64 data");
            return Err((StatusCode::BAD_REQUEST, Json(json_error("failed to decode base64 data"))));
        }
    };

    let mut parser = pathfinder4::Parser::new();
    let message = match parser.parse(&body_bytes) {
        Ok(message) => message,
        Err(err) => {
            tracing::error!(?err, "failed to parse data");
            return Err((StatusCode::BAD_REQUEST, Json(json_error(&format!("failed to parse data: {}", err)))));
        }
    };

    Ok(Json(serde_json::json!({
        "status": "ok",
        "text": payload.text,
        "voice_type": payload.voice_type,
        "format": payload.format,
        "parsed": format!("{:#?}", message),
        "warnings": parser.warnings,
    })))
}

async fn handle_log_text(Json(payload): Json<LogTextRequest>) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!(?payload, "received log_text request");
    let body_bytes = match BASE64_STANDARD.decode(&payload.data) {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!(?err, "failed to decode base64 body");
            return Err((StatusCode::BAD_REQUEST, Json(json_error("failed to decode base64 body"))));
        }
    };

    let save_path = "log_text.txt";
    let save_line = format!("{},{}\n", payload.text, payload.data);

    if let Err(err) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&save_path)
        .and_then(|mut file| std::io::Write::write_all(&mut file, save_line.as_bytes()))
    {
        tracing::error!(?err, %save_path, "failed to append log_text line to file");
    }

    print!("{}", pathfinder4::dump_proto(&body_bytes, 0).unwrap());
    let mut parser = pathfinder4::Parser::new();
    let message = parser.parse(&body_bytes).unwrap();
    println!("Parsed message: {:?}", message);

    Ok(Json(serde_json::json!({
        "status": "success",
        "log_text_file": save_path,
    })))
}

async fn handle_tts_request(State(state): State<Arc<AppState>>, Json(payload): Json<TtsRequest>) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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

    let wav_body = match crate::voices::text_to_speech(
        SpeechText::DynamicText(payload.text.clone(), Voice::VOICEVOX(voice_id)),
        &state.voicevox,
    )
    .await
    {
        Ok(b) => b,
        Err(err) => {
            tracing::error!(?err, "voicevox synth failed");
            return Err((StatusCode::BAD_GATEWAY, Json(json_error("failed to synthesize audio"))));
        }
    };

    Ok((
        StatusCode::OK,
        [("Content-Type", "audio/wav")],
        wav_body,
    ))
}

fn map_voice_type(voice_type: &str) -> Option<u32> {
    let voices = HashMap::from([
        ("standard", 1),
        ("soft", 2),
        ("bright", 3),
    ]);

    voices.get(voice_type).copied()
}

async fn handle_voicevox_speakers(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Speaker>>, (StatusCode, Json<serde_json::Value>)> {
    let speakers = match state.voicevox.speakers().await {
        Ok(list) => list,
        Err(err) => {
            tracing::error!(?err, "failed to fetch voicevox speakers");
            return Err((StatusCode::BAD_GATEWAY, Json(json_error("failed to fetch speakers"))));
        }
    };

    Ok(Json(speakers))
}

async fn handle_scenarios(State(state): State<Arc<AppState>>) -> Json<Vec<String>> {
    let names = state.scenarios.keys().cloned().collect::<Vec<_>>();
    Json(names)
}

fn json_error(message: &str) -> serde_json::Value {
    serde_json::json!({ "error": message })
}
