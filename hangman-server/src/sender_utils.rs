use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};
use tokio::sync::{mpsc, mpsc::error::SendError};
use tracing::warn;

#[async_trait]
pub trait LogSend<Item> {
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

#[async_trait]
pub trait SendToAll<M> {
    async fn send_to_all(self, msg: M);
}

#[async_trait]
impl<'a, M, I> SendToAll<M> for I
where
    M: Clone + Send + 'static,
    I: Iterator<Item = &'a mpsc::Sender<M>> + Send,
{
    async fn send_to_all(self, msg: M) {
        let mut futs: FuturesUnordered<_> = self
            .map(|s| {
                let msg = msg.clone();
                async {
                    if let Err(e) = s.send(msg).await {
                        warn!("failed to send message to socket: {e}");
                    }
                }
            })
            .collect();
        while let Some(_) = futs.next().await {}
    }
}
