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

type Players = HashMap<UserToken, (User, mpsc::Sender<ServerMessage>)>;

pub async fn game_logic(game: ServerGame, mut rx: mpsc::Receiver<GameMessage>) {
    // Game logic
    let code = game.code;
    let mut players = Players::new();
    let mut chat: Vec<(String, String)> = vec![];
    let tries_used = 0;

    while let Some(msg) = rx.recv().await {
        info!("[{code}] received {msg:?}");
        match msg {
            GameMessage::Join { user, sender } => {
                info!("[{code}] {:?} joins the game", user);
                let sender_c = sender.clone();
                let token = user.token;
                players.insert(user.token, (user, sender));
                let settings = game.settings.clone();
                let player_names = player_names(&players);
                sender_c
                    .log_send(ServerMessage::Init(Game {
                        settings,
                        players: player_names.clone(),
                        chat: chat.clone(),
                        tries_used,
                    }))
                    .await;
                // Send update to all clients
                // TODO: Replace with send_to_all
                for (_, (_, sender)) in players.iter().filter(|(t, _)| **t != token) {
                    sender
                        .log_send(ServerMessage::UpdatePlayers(player_names.clone()))
                        .await;
                }
            }
            GameMessage::Leave(token) => {
                if let Some((user, _)) = players.remove(&token) {
                    info!("[{code}] {:?} left the game", user);
                } else {
                    warn!("there was no user in this game with this token");
                }
                // Send update to all clients
                let player_names = player_names(&players);
                // TODO: Replace with send_to_all
                for (_, (_, sender)) in &players {
                    sender
                        .log_send(ServerMessage::UpdatePlayers(player_names.clone()))
                        .await;
                }
            }
            GameMessage::ClientMessage {
                message: ClientMessage::ChatMessage(msg),
                token,
            } => {
                if let Some((user, _)) = &players.get(&token) {
                    let message = (
                        user.nickname.clone(),
                        msg,
                    );
                    chat.push(message.clone());
                    // TODO: Replace with send_to_all
                    for (_, (_, sender)) in &players {
                        sender
                            .log_send(ServerMessage::ChatMessage(message.clone()))
                            .await;
                    }
                }
            }
        }
    }
}

fn player_names(players: &Players) -> Vec<String> {
    players.values().map(|(u, _)| u.nickname.clone()).collect()
}
