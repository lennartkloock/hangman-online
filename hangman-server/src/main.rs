use crate::{game::GameManager, word_generator::WordGenerator};
use axum::{
    routing::{get, post},
    Router,
};
use hangman_data::GameLanguage;
use std::net::SocketAddr;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{debug, info};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};
use crate::word_generator::GENERATOR;

mod api;
mod config;
mod game;
mod sender_utils;
mod word_generator;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    debug!("loading config");
    let config = config::load_config();

    GENERATOR
        .set(
            WordGenerator::preprocess(&config, &GameLanguage::all())
                .await
                .expect("failed to preprocess wordlists"),
        )
        .expect("failed to set global state");

    info!("starting hangman server on port {}", config.port);
    let app = Router::new()
        .route("/api/game", post(api::create_game))
        .route("/api/game/:code/ws", get(api::game_ws))
        .fallback_service(
            ServeDir::new(&config.public_dir)
                .not_found_service(ServeFile::new(format!("{}/index.html", config.public_dir))),
        )
        .with_state(GameManager::new())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::new(config.address, config.port);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to open server");
}
