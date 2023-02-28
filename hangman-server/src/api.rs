use crate::game::{Game, GameManagerState};
use axum::{
    extract::{ws::WebSocket, ConnectInfo, Path, State, WebSocketUpgrade},
    http::StatusCode,
    Json,
};
use hangman_data::{GameCode, GameSettings};
use std::net::SocketAddr;
use tracing::info;

pub async fn create_game(
    State(game_manager): State<GameManagerState>,
    Json(settings): Json<GameSettings>,
) -> StatusCode {
    let game = Game::new(settings);
    game_manager.lock().await.add_game(game);
    StatusCode::CREATED
}

pub async fn game_ws(
    State(game_manager): State<GameManagerState>,
    Path(code): Path<GameCode>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) {
    ws.on_upgrade(move |s| handle_socket(s, addr, code));
}

async fn handle_socket(mut socket: WebSocket, addr: SocketAddr, code: GameCode) {
    info!("new ws connection by {addr} for game {code}");
}
