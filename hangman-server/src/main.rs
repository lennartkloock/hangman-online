use crate::game::GameManager;
use axum::{
    routing::{get, post},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

mod api;
mod game;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    info!("starting hangman server");
    let app = Router::new()
        .route("/api/game", post(api::create_game))
        .route("/api/game/:code/ws", get(api::game_ws))
        .fallback_service(
            ServeDir::new("hangman-web/dist")
                .not_found_service(ServeFile::new("hangman-web/dist/index.html")),
        )
        .with_state(Arc::new(Mutex::new(GameManager::default())))
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("failed to open server");
}
