use axum::{http::StatusCode, response::{Html, IntoResponse}, routing::{get, post}, Json, Router};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use base64::prelude::*;
mod pathfinder4;
mod canned_message;
mod scenarios;
mod voices;
use crate::{canned_message::Canned, pathfinder4::parse, scenarios::{SharedScenario, build_scenarios}, voices::{VoicevoxClient, VoiceProviders, StaticVoiceRepository, format_speech_text}};
use axum::extract::State;

#[derive(Clone)]
struct AppState {
    providers: VoiceProviders,
    scenarios: HashMap<String, SharedScenario>,
}

#[derive(Debug, Deserialize)]
struct TtsRequest {
    text: String,
    data: String,
    scenario_name: String,
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
    let scenarios = build_scenarios(&supported_speakers);

    let providers = VoiceProviders {
        static_voices: StaticVoiceRepository::new("./static_voices"),
        voicebox: Some(voicevox),
    };
    let state = Arc::new(AppState {
        providers,
        scenarios,
    });

    let app = Router::new()
        .route("/", get(index_html))
        .route("/test", get(test_html))
        .route("/sample.json", get(sample_json))
        .route("/parse", post(handle_parse_request))
        .route("/tts", post(handle_tts_request))
        .route("/scenarios", get(handle_scenarios))
        .route("/canned_messages/:scenario_name", get(handle_canned_messages))
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

async fn handle_parse_request(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TtsRequest>
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let scenario = match state.scenarios.get(&payload.scenario_name) {
        Some(s) => s,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json_error(&format!("unknown scenario: {}", payload.scenario_name))),
            ));
        }
    };

    let body_bytes = match BASE64_STANDARD.decode(&payload.data) {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!(?err, "failed to decode base64 data");
            return Err((StatusCode::BAD_REQUEST, Json(json_error("failed to decode base64 data"))));
        }
    };

    let message = match parse(&body_bytes) {
        Ok(message) => message,
        Err(err) => {
            tracing::error!(?err, "failed to parse data");
            return Err((StatusCode::BAD_REQUEST, Json(json_error(&format!("failed to parse data: {}", err)))));
        }
    };

    let input = crate::scenarios::Input::new(&payload.text, Some(&message), &payload.data);

    let speech_text = scenario.render(input);

    Ok(Json(serde_json::json!({
        "status": "ok",
        "text": payload.text,
        "parsed": format!("{:#?}", message),
        "render_result": format_speech_text(&speech_text),
    })))
}

async fn handle_tts_request(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TtsRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if payload.text.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json_error("text must not be empty"))));
    }

    let scenario = match state.scenarios.get(&payload.scenario_name) {
        Some(s) => s,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json_error(&format!("unknown scenario: {}", payload.scenario_name))),
            ));
        }
    };

        let body_bytes = match BASE64_STANDARD.decode(&payload.data) {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!(?err, "failed to decode base64 data");
            return Err((StatusCode::BAD_REQUEST, Json(json_error("failed to decode base64 data"))));
        }
    };

    let message = match parse(&body_bytes) {
        Ok(message) => message,
        Err(err) => {
            tracing::error!(?err, "failed to parse data");
            return Err((StatusCode::BAD_REQUEST, Json(json_error(&format!("failed to parse data: {}", err)))));
        }
    };

    let input = crate::scenarios::Input::new(&payload.text, Some(&message), &payload.data);

    let speech_text = scenario.render(input);

    let wav_body = match crate::voices::text_to_speech(speech_text, &state.providers).await {
        Ok(b) => b,
        Err(err) => {
            tracing::error!(?err, "voicevox synth failed");
            return Err((StatusCode::BAD_GATEWAY, Json(json_error("failed to synthesize audio"))));
        }
    };

    Ok((StatusCode::OK, [("Content-Type", "audio/wav")], wav_body))
}

async fn handle_scenarios(State(state): State<Arc<AppState>>) -> Json<Vec<String>> {
    let mut names = state.scenarios.keys().cloned().collect::<Vec<_>>();
    names.sort();
    Json(names)
}

async fn handle_canned_messages(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(scenario_name): axum::extract::Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let scenario = match state.scenarios.get(&scenario_name) {
        Some(s) => s,
        None => return Err((
            StatusCode::BAD_REQUEST,
            Json(json_error(&format!("unknown scenario: {}", scenario_name))),
        )),
    };

    let mut map = std::collections::HashMap::new();
    for &c in Canned::ALL {
        let speech_text = scenario.render_canned_message(c);
        match crate::voices::text_to_speech(speech_text, &state.providers).await {
            Ok(wav) => { map.insert(c, bytes::Bytes::from(wav)); }
            Err(err) => tracing::warn!(?err, filename = c.filename(), "skipping canned message"),
        }
    }

    let zip = crate::canned_message::build_canned_message(map)
        .map_err(|err| {
            tracing::error!(?err, "failed to build canned message zip");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error("failed to build zip")))
        })?;

    Ok((StatusCode::OK, [("Content-Type", "application/zip")], zip))
}

fn json_error(message: &str) -> serde_json::Value {
    serde_json::json!({ "error": message })
}
