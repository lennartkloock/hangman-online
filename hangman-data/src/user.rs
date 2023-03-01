use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
};
use rand::Rng;
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

pub struct UserTokenAuthHeader(pub UserToken);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for UserTokenAuthHeader {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match parts
            .headers
            .get(header::AUTHORIZATION)
            .map(|auth| auth.to_str().map(|s| UserToken::from_str(s)))
        {
            Some(Ok(Ok(token))) => Ok(Self(token)),
            Some(Err(_)) => Err((
                StatusCode::BAD_REQUEST,
                "`Authorization` header contains invalid characters",
            )),
            Some(Ok(Err(_))) => Err((
                StatusCode::BAD_REQUEST,
                "`Authorization` header must contain a valid user token",
            )),
            None => Err((StatusCode::BAD_REQUEST, "`Authorization` header is missing")),
        }
    }
}
