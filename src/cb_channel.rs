use async_std::channel::{self, Receiver, RecvError, SendError, Sender};

#[derive(Debug)]
pub struct Replier<R> {
    reply: Sender<R>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReturningSenderError<T> {
    SendError(T),
    ReplyError,
}

#[derive(Debug, Clone)]
pub struct ReturningSender<T, R> {
    sender: Sender<(T, Replier<R>)>,
}

#[derive(Debug)]
pub struct ReturningReceiver<T, R> {
    receiver: Receiver<(T, Replier<R>)>,
}

pub fn bounded<T, R>(buffer: usize) -> (ReturningSender<T, R>, ReturningReceiver<T, R>) {
    let (sender, receiver) = channel::bounded(buffer);
    let tx = ReturningSender { sender };
    let rx = ReturningReceiver { receiver };
    (tx, rx)
}

pub fn unbounded<T, R>() -> (ReturningSender<T, R>, ReturningReceiver<T, R>) {
    let (sender, receiver) = channel::unbounded();
    let tx = ReturningSender { sender };
    let rx = ReturningReceiver { receiver };
    (tx, rx)
}

impl<T, R> ReturningSender<T, R> {
    pub async fn send(&self, msg: T) -> Result<R, ReturningSenderError<T>> {
        let (ostx, osrx) = channel::bounded(1);
        let ostx = Replier { reply: ostx };

        let inner_msg = (msg, ostx);
        self.sender.send(inner_msg).await.map_err(|e| {
            let t = e.into_inner();
            ReturningSenderError::SendError(t.0)
        })?;

        osrx.recv()
            .await
            .map_err(|_| ReturningSenderError::ReplyError)
    }

    pub fn close(&self) -> bool {
        self.sender.close()
    }
}

impl<T, R> ReturningReceiver<T, R> {
    pub async fn recv(&self) -> Result<(T, Replier<R>), RecvError> {
        self.receiver.recv().await
    }
}

impl<R> Replier<R> {
    pub async fn reply(self, msg: R) -> Result<(), SendError<R>> {
        self.reply.send(msg).await
    }
}
