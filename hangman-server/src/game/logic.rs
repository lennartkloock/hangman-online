//! Game logic

use crate::game::Game;
use hangman_data::{ClientMessage, ServerMessage, User, UserToken};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, info};

#[derive(Debug)]
pub enum GameMessage {
    ClientMessage {
        token: UserToken,
        message: ClientMessage,
    },
    Join {
        user: User,
        sender: mpsc::Sender<ServerMessage>,
    },
}

pub async fn game_logic(game: Game, mut rx: mpsc::Receiver<GameMessage>) {
    // Game logic
    let code = game.code;
    let mut players = HashMap::new();

    while let Some(msg) = rx.recv().await {
        info!("[{code}] received {msg:?}");
        match msg {
            GameMessage::Join { user, sender } => {
                info!("[{code}] {:?} joins the game", user);
                players.insert(user.token, (user, sender));
            }
            GameMessage::ClientMessage { message, .. } => match message {
                ClientMessage::Ping => {
                    for (user, tx) in players.values() {
                        debug!("Sending Pong to {user:?}");
                        tx.send(ServerMessage::Pong).await.unwrap();
                    }
                }
            },
        }
    }
}
