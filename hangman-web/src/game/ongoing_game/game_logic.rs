use crate::game::ongoing_game::GameState;
use dioxus::prelude::*;
use hangman_data::{GameLanguage, GameSettings, ServerMessage};

pub fn handle_message(msg: ServerMessage, state: &UseState<GameState>) {
    match msg {
        ServerMessage::Init { players } => state.set(GameState::Joined {
            settings: GameSettings {
                language: GameLanguage::English,
            },
            players,
            chat: vec![
                ("PockelHockel".to_string(), "Hello".to_string()),
                ("Testuser".to_string(), "Hello!".to_string()),
                ("Testuserwad".to_string(), "Hello World!".to_string()),
            ],
            tries_used: 3,
        }),
    }
}
