use crate::{
    components::{CenterContainer, Error},
    game::{create_user::CreateUser, ongoing_game::OngoingGame},
    storage,
    storage::{StorageError, User},
};
use dioxus::prelude::*;
use dioxus_router::use_route;
use fermi::{use_read, Atom};
use std::{
    convert::Infallible,
    fmt::{Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};
use thiserror::Error;

mod create_user;
mod ongoing_game;

/// Two bytes that represent a game code
///
/// 4 characters encoded in hex
#[derive(PartialEq)]
pub struct GameCode(u16);

#[derive(Debug, Error)]
pub enum ParseGameCodeError {
    #[error("game code must be 4 characters long")]
    TooShort,
    #[error("invalid game code: {0}")]
    ParseIntError(#[from] ParseIntError),
}

impl FromStr for GameCode {
    type Err = ParseGameCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err(ParseGameCodeError::TooShort);
        }
        u16::from_str_radix(s, 16)
            .map(Self)
            .map_err(ParseGameCodeError::from)
    }
}

impl Display for GameCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04X}", self.0)
    }
}

static USER: Atom<Result<Option<User>, StorageError>> = |_| storage::load::<User>("hangman_user");

pub fn Game(cx: Scope) -> Element {
    let route = use_route(cx);

    let code = route.parse_segment::<GameCode>("code");
    let user = use_read(cx, USER);

    match (code, user) {
        // Render game
        (Some(Ok(code)), Ok(Some(_user))) => cx.render(rsx!(OngoingGame { code: code })),

        // Invalid game code
        (Some(Err(e)), _) => cx.render(rsx!(CenterContainer {
            Error {
                title: "Invalid code",
                error: e
            }
        })),
        // No game code found
        (None, _) => cx.render(rsx!(CenterContainer {
            // Any type that implements Error
            Error::<Infallible> {
                title: "No code"
            }
        })),

        // No saved user
        (_, Ok(None)) => cx.render(rsx!(CreateUser {})),
        // Erroneous user
        (_, Err(e)) => cx.render(rsx!(CenterContainer { Error {
            title: "Failed to load user",
            error: e
        }})),
    }
}
