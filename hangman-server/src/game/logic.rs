//! Game logic

use crate::sender_utils::send_to_all;
use hangman_data::{ChatMessage, ClientMessage, CompetitiveState, ServerMessage, TeamState, User, UserToken};
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
    Team(GameMessageInner<TeamState>),
    Competitive(GameMessageInner<CompetitiveState>),
}

#[derive(Debug)]
pub enum GameMessageInner<S> {
    Join {
        user: User,
        sender: mpsc::Sender<ServerMessage<S>>,
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
pub struct Players<S>(HashMap<UserToken, (mpsc::Sender<ServerMessage<S>>, User)>);

impl<S: Debug + Clone> Players<S> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub async fn add_player(&mut self, tx: mpsc::Sender<ServerMessage<S>>, user: User) {
        self.insert(user.token, (tx, user));
    }

    pub async fn remove_player(
        &mut self,
        token: &UserToken,
    ) -> Option<(mpsc::Sender<ServerMessage<S>>, User)> {
        self.remove(token)
    }

    pub async fn send_to_all(&self, msg: ServerMessage<S>) {
        send_to_all(self.iter().map(|(_, (s, _))| s), msg).await;
    }

    pub fn player_names(&self) -> Vec<String> {
        self.values().map(|(_, u)| u.nickname.clone()).collect()
    }
}

impl<S> Deref for Players<S> {
    type Target = HashMap<UserToken, (mpsc::Sender<ServerMessage<S>>, User)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> DerefMut for Players<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
