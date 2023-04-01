use crate::game::ongoing_game::ClientState;
use dioxus::prelude::*;
use hangman_data::ServerMessage;

pub fn handle_message(msg: ServerMessage, state: &UseRef<ClientState>) {
    match msg {
        ServerMessage::UpdateGame(game) => state.set(ClientState::Joined(game)),
    }
}
