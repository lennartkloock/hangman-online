//! Game logic

use crate::sender_utils::send_to_all;
use hangman_data::{ChatMessage, ClientMessage, ServerMessage, User, UserToken};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tokio::sync::{mpsc, RwLock};

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
        let mut names = self.player_names();
        names.push(user.nickname.clone());
        self.send_to_all(ServerMessage::UpdatePlayers(names)).await;
        self.insert(user.token, (tx, user));
    }

    pub async fn remove_player(
        &mut self,
        token: &UserToken,
    ) -> Option<(mpsc::Sender<ServerMessage>, User)> {
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

pub struct Chat {
    players: Arc<RwLock<Players>>,
    messages: Vec<ChatMessage>,
}

impl Chat {
    pub fn new(players: Arc<RwLock<Players>>) -> Self {
        Self {
            players,
            messages: vec![],
        }
    }

    pub async fn send_message(&mut self, msg: ChatMessage) {
        self.messages.push(msg.clone());
        self.players
            .read()
            .await
            .send_to_all(ServerMessage::ChatMessage(msg))
            .await;
    }
}

impl Deref for Chat {
    type Target = Vec<ChatMessage>;

    fn deref(&self) -> &Self::Target {
        &self.messages
    }
}

impl DerefMut for Chat {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.messages
    }
}
