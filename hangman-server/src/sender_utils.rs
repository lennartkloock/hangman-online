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

// TODO
// #[async_trait]
// pub trait SendToAll<M> {
//     type E;
//
//     async fn send_to_all(self, msg: M) -> Option<Self::E>;
// }
//
// #[async_trait]
// impl<M, I> SendToAll<M> for I
// where
//     M: Send + Clone + Sync,
//     I: Iterator<Item = mpsc::Sender<M>> + Send,
// {
//     type E = SendError<M>;
//
//     async fn send_to_all(self, msg: M) -> Option<Self::E> {
//         for s in self {
//             if let Err(e) = s.send(msg.clone()).await {
//                 warn!("failed to send message to socket: {e}");
//                 return Some(e);
//             }
//         }
//         None
//     }
// }
