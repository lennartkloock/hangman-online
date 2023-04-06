use crate::game::ongoing_game::{ClientState, ConnectionError};
use dioxus::prelude::*;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use gloo_net::websocket::{futures::WebSocket, Message, WebSocketError};
use hangman_data::{ClientMessage, ServerMessage};
use log::debug;

pub fn connect(
    state: &UseRef<ClientState>,
    url: String,
) -> (
    Option<SplitSink<WebSocket, Message>>,
    Option<SplitStream<WebSocket>>,
) {
    match WebSocket::open(&url) {
        Ok(ws) => {
            let ws = ws.split();
            (Some(ws.0), Some(ws.1))
        }
        Err(e) => {
            debug!("failed to connect to socket");
            state.set(ClientState::Error(ConnectionError::SyntaxError(e).rc()));
            (None, None)
        }
    }
}

pub async fn ws_read(ws_rx: Option<SplitStream<WebSocket>>, state: UseRef<ClientState>) {
    if let Some(mut ws_read) = ws_rx {
        while let Some(msg) = ws_read.next().await {
            match msg {
                Ok(Message::Text(s)) => match serde_json::from_str::<ServerMessage>(&s) {
                    Ok(msg) => state.set(msg.into()),
                    Err(e) => state.set(ClientState::Error(
                        ConnectionError::DeserializeError(e).rc(),
                    )),
                },
                Ok(_) => {
                    state.set(ClientState::Error(
                        ConnectionError::DeserializeWrongDataTypeError.rc(),
                    ));
                }
                Err(WebSocketError::ConnectionClose(gloo_net::websocket::events::CloseEvent {
                    code: 4000,
                    ..
                })) => {
                    state.set(ClientState::Error(ConnectionError::GameNotFound.rc()));
                }
                Err(WebSocketError::ConnectionClose(gloo_net::websocket::events::CloseEvent {
                    code: 4001,
                    ..
                })) => {
                    state.set(ClientState::Error(ConnectionError::GameClosed.rc()));
                }
                Err(e) => {
                    state.set(ClientState::Error(ConnectionError::WsError(e).rc()));
                }
            }
        }
    }
}

pub async fn ws_write(
    mut rx: UnboundedReceiver<ClientMessage>,
    ws_tx: Option<SplitSink<WebSocket, Message>>,
    state: UseRef<ClientState>,
) {
    if let Some(mut ws_write) = ws_tx {
        while let Some(msg) = rx.next().await {
            match serde_json::to_string(&msg) {
                Ok(s) => match ws_write.send(Message::Text(s)).await {
                    Err(WebSocketError::MessageSendError(e)) => {
                        state.set(ClientState::Error(ConnectionError::SendError(e).rc()))
                    }
                    Err(e) => state.set(ClientState::Error(ConnectionError::WsError(e).rc())),
                    Ok(_) => {}
                },
                Err(e) => state.set(ClientState::Error(ConnectionError::SerializeError(e).rc())),
            }
        }
    }
}
