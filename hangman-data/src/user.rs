use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{
    fmt::{Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};

#[derive(Copy, Clone, Debug, DeserializeFromStr, Eq, Hash, SerializeDisplay, PartialEq)]
pub struct UserToken(u64);

impl UserToken {
    pub fn random() -> Self {
        Self(rand::thread_rng().gen())
    }
}

impl FromStr for UserToken {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str_radix(s, 16).map(Self)
    }
}

impl Display for UserToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct User {
    pub nickname: String,
    pub token: UserToken,
}

impl User {
    pub fn new(nickname: &str) -> Self {
        Self {
            nickname: nickname.to_string(),
            token: UserToken::random(),
        }
    }
}
