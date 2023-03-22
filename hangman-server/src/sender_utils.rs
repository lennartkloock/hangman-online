use std::fmt::Debug;
use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};
use tokio::sync::{mpsc, mpsc::error::SendError};
use tracing::debug;

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
            debug!("failed to send message to socket: {e}");
            Some(e)
        } else {
            None
        }
    }
}

pub async fn send_to_all<'a, I: Iterator<Item = &'a mpsc::Sender<M>>, M: Debug + Clone + 'a>(iter: I, msg: M) {
    debug!("sending {msg:?} to all");
    let mut futs: FuturesUnordered<_> = iter
        .map(|s| {
            let msg = msg.clone();
            async {
                if let Err(e) = s.send(msg).await {
                    debug!("failed to send message to socket: {e}");
                }
            }
        })
        .collect();
    while (futs.next().await).is_some() {}
}
