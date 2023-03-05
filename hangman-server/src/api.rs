use crate::game::{Game, GameManagerState, GameMessage};
use axum::{
    extract::{
        ws::{CloseFrame, Message, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::{SinkExt, StreamExt};
use hangman_data::{CreateGameBody, GameCode, User};
use std::borrow::Cow;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

pub async fn create_game(
    State(game_manager): State<GameManagerState>,
    Json(CreateGameBody { token, settings }): Json<CreateGameBody>,
) -> (StatusCode, Json<GameCode>) {
    let game = Game::new(token, settings);
    let code = game.code;
    game_manager.lock().await.add_game(game);
    (StatusCode::CREATED, Json(code))
}

pub async fn game_ws(
    State(game_manager): State<GameManagerState>,
    Path(code): Path<GameCode>,
    Query(user): Query<User>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    if let Some(game_socket) = game_manager.lock().await.get_game(code) {
        ws.on_upgrade(move |socket| handle_socket(socket, user, code, game_socket))
    } else {
        ws.on_upgrade(move |mut socket| async move {
            if let Err(e) = socket
                .send(Message::Close(Some(CloseFrame {
                    code: 4000,
                    reason: Cow::from("game not found"),
                })))
                .await
            {
                warn!("game not found but failed to send close frame to player socket: {e}");
            }
        })
    }
}

async fn handle_socket(
    socket: WebSocket,
    user: User,
    code: GameCode,
    game_socket: mpsc::Sender<GameMessage>,
) {
    info!("new ws connection by {} for game {code}", user.nickname);
    let (mut sender, mut receiver) = socket.split();

    // Send out messages sent to the internal client socket
    let (tx, mut rx) = mpsc::channel(1);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            info!("sending {msg:?} to client {}", user.token);
            match serde_json::to_string(&msg) {
                Ok(t) => {
                    if let Err(e) = sender.send(Message::Text(t)).await {
                        warn!("failed to send ws message to client: {e}");
                        break;
                    }
                }
                Err(e) => {
                    error!("failed to serialize message: {e}");
                    break;
                }
            }
        }
        if let Err(e) = sender
            .send(Message::Close(Some(CloseFrame {
                code: 4001,
                reason: Cow::from("the game ended"),
            })))
            .await
        {
            warn!("game ended but failed to send close frame to player socket: {e}");
        }
    });

    // Copy user token
    let token = user.token;

    // Join Game
    game_socket
        .send(GameMessage::Join { user, sender: tx })
        .await
        .unwrap();

    tokio::spawn(async move {
        // Parse and send client messages to game socket
        while let Some(msg) = receiver.next().await {
            match msg.map(|m| m.to_text().map(serde_json::from_str)) {
                Ok(Ok(Ok(message))) => {
                    if let Err(e) = game_socket
                        .send(GameMessage::ClientMessage { token, message })
                        .await
                    {
                        error!("failed to send message to game socket: {e}");
                        break;
                    }
                }
                Ok(Ok(Err(e))) => warn!("failed to parse ws message: {e}"),
                Ok(Err(e)) => warn!("failed to parse ws message as text: {e}"),
                Err(e) => warn!("failed to receive ws message: {e}"),
            }
        }
    });
}
