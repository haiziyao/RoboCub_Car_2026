use super::model::WebMessage;
use super::state::WebState;
use crate::embed::Assets;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use log::{debug, info};

pub async fn index() -> impl IntoResponse {
    info!("Index getting started");
    match Assets::get("index.html") {
        Some(file) => {
            Html(String::from_utf8_lossy(file.data.as_ref()).into_owned()).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn message(State(state): State<WebState>) -> impl IntoResponse {
    // info!("sending cached message");

    match state.latest().await {
        Some(msg) => Json(msg).into_response(),
        None => Json(WebMessage::empty()).into_response(),
    }
}

pub async fn push_message(
    State(state): State<WebState>,
    Json(msg): Json<WebMessage>,
) -> impl IntoResponse {
    info!("accepting pushed web message");
    state.push_message(msg).await;
    StatusCode::ACCEPTED.into_response()
}

pub async fn history(State(state): State<WebState>) -> impl IntoResponse {
    // info!("sending cached history");

    Json(state.history().await).into_response()
}

pub async fn handle_404() -> impl IntoResponse {
    debug!("not found! 404 problem");
    (StatusCode::NOT_FOUND, "Not found")
}
