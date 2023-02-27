use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    id: String,
    nickname: String,
}

impl User {
    pub fn new(nickname: &str) -> Self {
        let id = rand::thread_rng().next_u64();
        Self {
            id: format!("{id:016x}"),
            nickname: nickname.to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("failed to retrieve window object")]
    NoWindow,
    #[error("failed to retrieve local storage object")]
    NoLocalStorage,
    #[error("failed to serialize data: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("failed to store/load data to/from storage")]
    StoreLoadError,
}

fn retrieve_local_storage() -> Result<web_sys::Storage, StorageError> {
    let window = web_sys::window().ok_or(StorageError::NoWindow)?;
    window.local_storage().ok().flatten().ok_or(StorageError::NoLocalStorage)
}

pub fn store<S: Serialize>(key: &str, data: &S) -> Result<(), StorageError> {
    let local_storage = retrieve_local_storage()?;
    local_storage.set_item(key, &serde_json::to_string(data)?).map_err(|_| StorageError::StoreLoadError)
}

pub fn load<T: DeserializeOwned>(key: &str) -> Result<Option<T>, StorageError> {
    let local_storage = retrieve_local_storage()?;
    if let Some(data) = local_storage.get_item(key).map_err(|_| StorageError::StoreLoadError)? {
        Ok(Some(serde_json::from_str(&data)?))
    } else {
        Ok(None)
    }
}
