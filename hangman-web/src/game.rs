use crate::components::Error;
use dioxus::prelude::*;
use dioxus_router::use_route;
use std::{
    fmt::{Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};
use std::convert::Infallible;

mod ongoing_game;

/// Two bytes that represent a game code
///
/// 4 characters encoded in hex
struct GameCode(u16);

#[derive(thiserror::Error, Debug)]
enum ParseGameCodeError {
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
        write!(f, "{}", self.0)
    }
}

pub fn Game(cx: Scope) -> Element {
    let route = use_route(cx);

    let code = route.parse_segment::<GameCode>("code");

    match code {
        Some(Ok(code)) => cx.render(rsx!(
            div {
                "Game {code}"
            }
        )),
        Some(Err(e)) => cx.render(rsx!(Error {
            title: "Invalid code",
            error: e
        })),
        None => cx.render(rsx!(Error::<Infallible> {// Any type that implements Error
            title: "No code"
        })),
    }
}
