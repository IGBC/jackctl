use crate::cb_channel::{self, ReturningReceiver, ReturningSender};
use crate::model2::events::{Event, JackCardAction, JackCmd};
use futures_lite::future::block_on;
use jack::AsyncClient;
use smol::{
    channel::{bounded, Receiver, Sender},
    LocalExecutor,
};
use std::thread;

/// An easily clonable handle to the jack runtime
#[derive(Clone)]
pub struct JackHandle {
    /// Send commands to the jack runtime
    cmd_tx: Sender<JackCmd>,
    /// Receive events from the jack runtime
    event_rx: Receiver<Event>,
    /// Send card actions to jack runtime with blocking ACK
    card_tx: ReturningSender<JackCardAction, ()>,
}

/// Jack server runtime and signalling state
pub struct JackRuntime {
    /// Connection to the jack server
    client: AsyncClient<(), ()>,
    /// Receive jack commands
    cmd_rx: Receiver<JackCmd>,
    /// Send events to the model layer
    event_tx: Sender<Event>,
    /// Receive card commands
    card_rx: ReturningReceiver<JackCardAction, ()>,
}

impl JackRuntime {
    pub fn start() -> JackHandle {
        let (event_tx, event_rx) = bounded(4);
        let (cmd_tx, cmd_rx) = bounded(4);
        let (card_tx, card_rx) = cb_channel::bounded(4);

        // Initialise and bootstrap the jack runtime
        Self {
            client: todo!(),
            cmd_rx,
            event_tx,
            card_rx,
        }
        .bootstrap();

        // Return a sending handle
        JackHandle {
            cmd_tx,
            event_rx,
            card_tx,
        }
    }

    /// Bootstrap a smol runtime on a dedicated thread
    fn bootstrap(self) {
        thread::spawn(move || {
            let rt_state = self;
            let local_exec = LocalExecutor::new();
            block_on(local_exec.run(rt_state.run()))
        });
    }

    /// Main executor run loop
    async fn run(mut self) {
        todo!()
    }
}
