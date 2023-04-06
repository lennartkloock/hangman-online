use crate::{
    game::{logic::GameMessage, GameManager},
    sender_utils::LogSend,
};
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
use hangman_data::{CreateGameBody, GameCode, GameMode, ServerMessage, User};
use std::borrow::Cow;
use std::fmt::Debug;
use futures::stream::SplitSink;
use serde::Serialize;
use tokio::sync::mpsc;
use tracing::{debug, error, trace, warn};
use tungstenite::Error;
use crate::game::logic::GameMessageInner;

pub async fn create_game(
    State(game_manager): State<GameManager>,
    Json(CreateGameBody { token, settings }): Json<CreateGameBody>,
) -> (StatusCode, Json<GameCode>) {
    let code = game_manager.add_game(token, settings).await;
    (StatusCode::CREATED, Json(code))
}

pub async fn game_ws(
    State(game_manager): State<GameManager>,
    Path(code): Path<GameCode>,
    Query(user): Query<User>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    if let Some(game) = game_manager.get_game(code).await {
        ws.on_upgrade(move |socket| handle_socket(socket, user, code, game))
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
    (mode, game_socket): (GameMode, mpsc::Sender<GameMessage>),
) {
    debug!("new ws connection by {} for game {code}", user.nickname);
    let (sender, mut receiver) = socket.split();

    // Copy user token
    let token = user.token;

    // Join Game
    let game_message = match &mode {
        GameMode::Team => {
            let tx = spawn_message_forwarder(sender, user.nickname.clone());
            GameMessage::Team(GameMessageInner::Join { user, sender: tx })
        },
        GameMode::Competitive => {
            let tx = spawn_message_forwarder(sender, user.nickname.clone());
            GameMessage::Competitive(GameMessageInner::Join { user, sender: tx })
        },
    };
    game_socket
        .log_send(game_message)
        .await;

    // Task that parses and sends client messages to the game socket
    tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) => {
                    debug!("client sent closing frame");
                    let game_message = match mode {
                        GameMode::Team => GameMessage::Team(GameMessageInner::Leave(token)),
                        GameMode::Competitive => GameMessage::Competitive(GameMessageInner::Leave(token)),
                    };
                    game_socket.log_send(game_message).await;
                    break;
                }
                Ok(msg) => match msg.to_text().map(serde_json::from_str) {
                    Ok(Ok(message)) => {
                        let game_message = match &mode {
                            GameMode::Team => GameMessage::Team(GameMessageInner::ClientMessage { token, message }),
                            GameMode::Competitive => GameMessage::Competitive(GameMessageInner::ClientMessage { token, message }),
                        };
                        if game_socket
                            .log_send(game_message)
                            .await
                            .is_some()
                        {
                            break;
                        }
                    }
                    Ok(Err(e)) => warn!("failed to parse ws message: {e}"),
                    Err(e) => warn!("failed to parse ws message as text: {e}"),
                },
                Err(e) => {
                    let b = e
                        .into_inner()
                        .downcast::<Error>()
                        .expect("failed to downcast axum error to tungstenite error");
                    if let Error::Protocol(
                        tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,
                    ) = *b
                    {
                        debug!("client closed connection without closing frame");
                        let game_message = match &mode {
                            GameMode::Team => GameMessage::Team(GameMessageInner::Leave(token)),
                            GameMode::Competitive => GameMessage::Competitive(GameMessageInner::Leave(token)),
                        };
                        game_socket.log_send(game_message).await;
                        break;
                    } else {
                        warn!("failed to receive ws message: {}", *b);
                    }
                }
            }
        }
    });
}

// Todo: Too many bounds for State?
fn spawn_message_forwarder<State: Debug + Serialize + Send + Sync + 'static>(mut sender: SplitSink<WebSocket, Message>, nickname: String) -> mpsc::Sender<ServerMessage<State>> {
    // Send out messages sent to the internal client socket
    let (tx, mut rx) = mpsc::channel::<ServerMessage<State>>(1);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            trace!("sending {msg:?} to {}", nickname);
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
                reason: Cow::from("the game was closed"),
            })))
            .await
        {
            let b = e
                .into_inner()
                .downcast::<Error>()
                .expect("failed to downcast axum error to tungstenite error");
            if !matches!(*b, Error::ConnectionClosed) {
                // connection wasn't closed normally
                warn!("game ended but failed to send close frame to player socket: {b}");
            }
        }
    });
    tx
}
