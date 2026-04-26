mod errors;
mod handler;
mod models;
mod state;
use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post, put},
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new());
    let app = Router::new()
        .route("/flags", post(handler::create_flag))
        .route(
            "/flags/:key",
            get(handler::get_flag).patch(handler::toggle_flag),
        )
        .route("/flags/:key/overrides/:user_id", put(handler::set_override))
        .route("/flags/:key/evaluate", get(handler::evaluate_flag))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind TCP listener on 0.0.0.0:3000");

    println!("Server running on http://0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .expect("axum server failed");
}
