//! Game logic

use crate::game::Game;
use hangman_data::{ClientMessage, User, UserToken};
use tokio::sync::mpsc;
use tracing::info;

#[derive(Debug)]
pub enum GameMessage {
    ClientMessage {
        token: UserToken,
        message: ClientMessage,
    },
    JoinLobby(User),
}

pub async fn game_logic(game: Game, mut rx: mpsc::Receiver<GameMessage>) {
    // Game logic
    let code = game.code;
    let mut players = vec![];

    while let Some(msg) = rx.recv().await {
        info!("[{code}] received {msg:?}");
        match msg {
            GameMessage::JoinLobby(user) => {
                info!("[{code}] {} joins the game", user.nickname);
                players.push(user);
            }
            GameMessage::ClientMessage { .. } => {}
        }
    }
}
