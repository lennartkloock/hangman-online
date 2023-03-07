use async_trait::async_trait;
use tokio::sync::{mpsc, mpsc::error::SendError};
use tracing::warn;

#[async_trait]
pub trait LogSend<Item: Send> {
    type E;

    async fn log_send(&self, msg: Item) -> Option<Self::E>;
}

#[async_trait]
impl<Item: Send> LogSend<Item> for mpsc::Sender<Item> {
    type E = SendError<Item>;

    async fn log_send(&self, msg: Item) -> Option<Self::E> {
        if let Err(e) = self.send(msg).await {
            warn!("failed to send message to socket: {e}");
            Some(e)
        } else {
            None
        }
    }
}
