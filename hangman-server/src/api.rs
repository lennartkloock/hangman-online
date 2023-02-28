use crate::game::{Game, GameManagerState};
use axum::{
    extract::{ws::WebSocket, ConnectInfo, Path, State, WebSocketUpgrade},
    http::StatusCode,
    Json,
};
use hangman_data::{GameCode, GameSettings, UserToken};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::info;

#[derive(Deserialize)]
pub struct CreateGameBody {
    token: UserToken,
    settings: GameSettings,
}

#[derive(Serialize)]
pub struct CreateGameResponse {
    code: GameCode,
}

pub async fn create_game(
    State(game_manager): State<GameManagerState>,
    Json(CreateGameBody { token, settings }): Json<CreateGameBody>,
) -> (StatusCode, Json<CreateGameResponse>) {
    let game = Game::new(token, settings);
    let code = game.code;
    game_manager.lock().await.add_game(game);
    (StatusCode::CREATED, Json(CreateGameResponse { code }))
}

pub async fn game_ws(
    State(game_manager): State<GameManagerState>,
    Path(code): Path<GameCode>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> StatusCode {
    if let Some(_) = game_manager.lock().await.get_game(code) {
        ws.on_upgrade(move |s| handle_socket(s, addr, code));
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn handle_socket(mut socket: WebSocket, addr: SocketAddr, code: GameCode) {
    info!("new ws connection by {addr} for game {code}");
}
