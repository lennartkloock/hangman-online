//! Game logic

use crate::{
    game::{
        logic::word::{GuessResult, Word},
        ServerGame,
    },
    sender_utils::{LogSend, SendToAll},
};
use hangman_data::{
    ChatColor, ChatMessage, ClientMessage, Game, GameState, ServerMessage, User, UserToken,
};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

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
