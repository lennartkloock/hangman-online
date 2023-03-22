//! Game logic

use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use hangman_data::{ChatMessage, ClientMessage, ServerMessage, User, UserToken};
use crate::sender_utils::send_to_all;

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

pub trait ToName {
    fn to_name(&self) -> &str;
}

impl ToName for User {
    fn to_name(&self) -> &str {
        &self.nickname
    }
}

#[derive(Debug)]
pub struct Players<T>(HashMap<UserToken, (mpsc::Sender<ServerMessage>, T)>);

impl<T: ToName> Players<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub async fn add_player(&mut self, token: UserToken, tx: mpsc::Sender<ServerMessage>, t: T) {
        self.send_to_all(ServerMessage::UpdatePlayers(self.player_names())).await;
        self.insert(token, (tx, t));
    }

    pub async fn remove_player(
        &mut self,
        token: &UserToken,
    ) -> Option<(mpsc::Sender<ServerMessage>, T)> {
        let res = self.remove(token);
        if res.is_some() {
            self.send_to_all(ServerMessage::UpdatePlayers(self.player_names()))
                .await;
        }
        res
    }

    pub async fn send_to_all(&self, msg: ServerMessage) {
        send_to_all(self.iter().map(|(_, (s, _))| s), msg).await;
    }

    pub fn player_names(&self) -> Vec<String> {
        self.values()
            .map(|(_, t)| t.to_name().to_string())
            .collect()
    }
}

impl<T> Deref for Players<T> {
    type Target = HashMap<UserToken, (mpsc::Sender<ServerMessage>, T)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Players<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Chat<T> {
    players: Arc<RwLock<Players<T>>>,
    messages: Vec<ChatMessage>,
}

impl<T: ToName> Chat<T> {
    pub fn new(players: Arc<RwLock<Players<T>>>) -> Self {
        Self {
            players,
            messages: vec![],
        }
    }

    pub async fn send_message(&mut self, msg: ChatMessage) {
        self.messages.push(msg.clone());
        self.players.read().await.send_to_all(ServerMessage::ChatMessage(msg)).await;
    }

    pub async fn join_message(&mut self, name: &str) {
        self.send_message(ChatMessage {
            content: format!("→ {} joined the game", name),
            ..Default::default()
        }).await;
    }

    pub async fn leave_message(&mut self, name: &str) {
        self.send_message(ChatMessage {
            content: format!("← {} left the game", name),
            ..Default::default()
        }).await;
    }
}

impl<T> Deref for Chat<T> {
    type Target = Vec<ChatMessage>;

    fn deref(&self) -> &Self::Target {
        &self.messages
    }
}

impl<T> DerefMut for Chat<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.messages
    }
}
