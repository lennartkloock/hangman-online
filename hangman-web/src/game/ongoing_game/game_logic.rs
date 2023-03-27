use crate::game::ongoing_game::ClientState;
use dioxus::prelude::*;
use hangman_data::{Game, ServerMessage};
use log::{info, warn};

pub fn handle_message(msg: ServerMessage, state: &UseRef<ClientState>) {
    match msg {
        ServerMessage::Init(data) => state.set(ClientState::Joined(data)),
        ServerMessage::UpdatePlayers(p) => state.with_mut(|mut s| {
            info!("updating player list: {p:?}");
            if let ClientState::Joined(Game::InProgress { players, .. }) = &mut s {
                *players = p;
            } else {
                warn!("received update message before init");
            }
        }),
        ServerMessage::UpdateGame {
            word: w,
            tries_used: t,
        } => state.with_mut(|s| {
            info!("new guess: {w}");
            if let ClientState::Joined(Game::InProgress {
                word, tries_used, ..
            }) = s
            {
                *word = w;
                *tries_used = t;
            } else {
                warn!("received update message before init");
            }
        }),
        ServerMessage::ChatMessage(message) => state.with_mut(|s| {
            info!("new chat message: {message:?}");
            if let ClientState::Joined(Game::InProgress { chat, .. }) = s {
                chat.push(message);
            } else {
                warn!("received update message before init");
            }
        }),
        ServerMessage::UpdateGameState(game_state) => state.with_mut(|s| {
            info!("new game state: {game_state:?}");
            if let ClientState::Joined(Game::InProgress { state, .. }) = s {
                *state = game_state;
            } else {
                warn!("received update message before init");
            }
        }),
    }
}
