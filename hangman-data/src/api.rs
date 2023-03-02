use crate::{GameSettings, UserToken};
use serde::{Deserialize, Serialize};

mod ws;

pub use ws::*;

#[derive(Deserialize, Serialize)]
pub struct CreateGameBody {
    pub token: UserToken,
    pub settings: GameSettings,
}
