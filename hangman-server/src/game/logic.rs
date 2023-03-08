//! Game logic

use crate::{game::ServerGame, sender_utils::LogSend};
use hangman_data::{ClientMessage, Game, ServerMessage, User, UserToken};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{info, warn};

#[derive(Debug)]
pub enum GameMessage {
    Join {
        user: User,
        sender: mpsc::Sender<ServerMessage>,
    },
    Leave(UserToken),
    ClientMessage {
        token: UserToken,
        message: ClientMessage,
    },
}

pub async fn game_logic(game: ServerGame, mut rx: mpsc::Receiver<GameMessage>) {
    // Game logic
    let code = game.code;
    let mut players = HashMap::<UserToken, (User, mpsc::Sender<ServerMessage>)>::new();
    let mut chat: Vec<(String, String)> = vec![];
    let mut tries_used = 0;

    while let Some(msg) = rx.recv().await {
        info!("[{code}] received {msg:?}");
        match msg {
            GameMessage::Join { user, sender } => {
                info!("[{code}] {:?} joins the game", user);
                let sender_c = sender.clone();
                players.insert(user.token, (user, sender));
                let settings = game.settings.clone();
                sender_c
                    .log_send(ServerMessage::Init(Game {
                        settings,
                        players: players.values().map(|(u, _)| u.nickname.clone()).collect(),
                        chat: chat.clone(),
                        tries_used,
                    }))
                    .await;
            }
            GameMessage::Leave(token) => {
                if let Some((user, _)) = players.remove(&token) {
                    info!("[{code}] {:?} left the game", user);
                } else {
                    warn!("there was no user in this game with this token");
                }
                // Send update to all clients
            }
            GameMessage::ClientMessage { message, .. } => match message {},
        }
    }
}