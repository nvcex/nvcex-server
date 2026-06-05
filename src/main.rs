use axum::{http::StatusCode, response::{Html, IntoResponse}, routing::{get, post}, Json, Router};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use base64::prelude::*;
mod pathfinder4;

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
    let app = Router::new()
        .route("/", get(index_html))
        .route("/test", get(test_html))
        .route("/sample.json", get(sample_json))
        .route("/parse", post(handle_parse_request))
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

async fn handle_tts_request(Json(payload): Json<TtsRequest>) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if payload.text.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json_error("text must not be empty"))));
    }

    let _voice_id = match map_voice_type(&payload.voice_type) {
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

    let client = Client::new();

    let audio_query_url = "http://localhost:50000/audio_query";
    let audio_query_response = client
        .post(audio_query_url)
        .query(&[("text", payload.text.as_str()), ("speaker", "1"), ("enable_katakana_english", "true")])
        .header("Accept", "application/json")
        .send()
        .await;

    let audio_query_response = match audio_query_response {
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

    if !audio_query_response.status().is_success() {
        tracing::error!(status = ?audio_query_response.status(), "tts backend returned error");
        return Err(
            (
                StatusCode::BAD_GATEWAY,
                Json(json_error("TTS backend returned an error")),
            ),
        );
    }

    let audio_query_body = match audio_query_response.bytes().await {
        Ok(body) => body,
        Err(err) => {
            tracing::error!(?err, "failed to read body from TTS backend");
            return Err(
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json_error("failed to read body from TTS backend")),
                ),
            );
        }
    };

    let synthesis_url = "http://localhost:50000/synthesis";
    let synthesis_response = client
        .post(synthesis_url)
        .query(&[("speaker", "1"), ("enable_interrogative_upspeak", "true")])
        .header("Content-Type", "application/json")
        .body(audio_query_body)
        .send()
        .await;

    let synthesis_response = match synthesis_response {
        Ok(res) => res,
        Err(err) => {
            tracing::error!(?err, "failed to call synthesis endpoint");
            return Err(
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json_error("failed to communicate with TTS synthesis backend")),
                ),
            );
        }
    };

    if !synthesis_response.status().is_success() {
        tracing::error!(status = ?synthesis_response.status(), "tts synthesis backend returned error");
        return Err(
            (
                StatusCode::BAD_GATEWAY,
                Json(json_error("TTS synthesis backend returned an error")),
            ),
        );
    }

    let wav_body = match synthesis_response.bytes().await {
        Ok(body) => body,
        Err(err) => {
            tracing::error!(?err, "failed to read wav body from synthesis response");
            return Err(
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json_error("failed to read audio body from TTS backend")),
                ),
            );
        }
    };

    Ok((
        StatusCode::OK,
        [("Content-Type", "audio/wav")],
        wav_body,
    ))
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

#[derive(Debug, Deserialize)]
struct BackendResponse {
    source_url: String,
}
