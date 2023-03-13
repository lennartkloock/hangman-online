use hangman_data::{GameCode, User};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UrlError {
    #[error("failed to retrieve window")]
    NoWindow,
    #[error("js error")]
    JsError,
}

pub fn game_ws_url(code: &GameCode, user: &User) -> Result<String, UrlError> {
    match web_sys::window()
        .map(|w| w.location())
        .map(|l| (l.protocol(), l.host()))
    {
        Some((Ok(protocol), Ok(host))) => {
            let query = form_urlencoded::Serializer::new(String::new())
                .append_pair("nickname", &user.nickname)
                .append_pair("token", &format!("{}", user.token))
                .finish();
            let protocol = if protocol == "https:" { "wss:" } else { "ws:" };
            Ok(format!("{protocol}//{host}/api/game/{code}/ws?{query}"))
        }
        Some((_, _)) => Err(UrlError::JsError),
        None => Err(UrlError::NoWindow),
    }
}

pub fn http_url_origin() -> Result<String, UrlError> {
    match web_sys::window().map(|w| w.location()).map(|l| l.origin()) {
        Some(Ok(origin)) => Ok(origin),
        Some(Err(_)) => Err(UrlError::JsError),
        None => Err(UrlError::NoWindow),
    }
}
