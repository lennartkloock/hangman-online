use axum::Router;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    log::info!("starting hangman server");
    let app = Router::new()
        .fallback_service(
            ServeDir::new("hangman-web/dist")
                .not_found_service(ServeFile::new("hangman-web/dist/index.html")),
        )
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
