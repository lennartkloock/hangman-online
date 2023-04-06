//! Game logic

use crate::sender_utils::send_to_all;
use hangman_data::{ChatMessage, ClientMessage, ServerMessage, User, UserToken};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use std::fmt::Debug;
use tokio::sync::mpsc;

pub mod competitive;
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

pub fn join_message(name: &str) -> ChatMessage {
    ChatMessage {
        content: format!("→ {} joined the game", name),
        ..Default::default()
    }
}

pub fn leave_message(name: &str) -> ChatMessage {
    ChatMessage {
        content: format!("← {} left the game", name),
        ..Default::default()
    }
}

#[derive(Debug)]
pub struct Players(HashMap<UserToken, (mpsc::Sender<ServerMessage>, User)>);

impl Players {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub async fn add_player(&mut self, tx: mpsc::Sender<ServerMessage>, user: User) {
        self.insert(user.token, (tx, user));
    }

    pub async fn remove_player(
        &mut self,
        token: &UserToken,
    ) -> Option<(mpsc::Sender<ServerMessage>, User)> {
        self.remove(token)
    }

    pub async fn send_to_all(&self, msg: ServerMessage) {
        send_to_all(self.iter().map(|(_, (s, _))| s), msg).await;
    }

    pub fn player_names(&self) -> Vec<String> {
        self.values().map(|(_, u)| u.nickname.clone()).collect()
    }
}

impl Deref for Players {
    type Target = HashMap<UserToken, (mpsc::Sender<ServerMessage>, User)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Players {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
