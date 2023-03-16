//! Game logic

use hangman_data::{ClientMessage, ServerMessage, User, UserToken};
use tokio::sync::mpsc;

pub mod team;
mod word;

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
