mod async_client;
mod card;
mod client;
mod cmd;

use crate::cb_channel::{self, ReturningReceiver, ReturningSender};
use crate::model2::events::{Event, JackCardAction, JackCmd};
use async_std::{
    channel::{bounded, Receiver, Sender},
    task,
};
use jack::{AsyncClient, Client as JackClient, InternalClientID};
use std::sync::Arc;

use self::async_client::JackNotificationController;

/// An easily clonable handle to the jack runtime
#[derive(Clone)]
pub struct JackHandle {
    /// Send commands to the jack runtime
    cmd_tx: Sender<JackCmd>,
    /// Receive events from the jack runtime
    event_rx: Receiver<Event>,
    /// Send card actions to jack runtime with blocking ACK
    card_tx: ReturningSender<JackCardAction, Result<InternalClientID, jack::Error>>,
}

/// Jack server runtime and signalling state
pub struct JackRuntime {
    /// Async jack client
    a_client: AsyncClient<JackNotificationController, ()>,
    /// Receive jack commands
    cmd_rx: Receiver<JackCmd>,
    /// Send events to the model layer
    event_tx: Sender<Event>,
    /// Receive card commands
    card_rx: ReturningReceiver<JackCardAction, Result<InternalClientID, jack::Error>>,
}

impl JackRuntime {
    pub fn start() -> Result<JackHandle, jack::Error> {
        // Open the channels
        let (event_tx, event_rx) = bounded(4);
        let (cmd_tx, cmd_rx) = bounded(4);
        let (card_tx, card_rx) = cb_channel::bounded(4);

        // initialise jack
        let a_client = async_client::JackNotificationController::new(event_tx.clone());
        let (client, _) = JackClient::new("jackctl", jack::ClientOptions::NO_START_SERVER)?;
        let a_client = client.activate_async(a_client, ())?;

        // Initialise and bootstrap the jack runtime
        Arc::new(Self {
            a_client,
            cmd_rx,
            event_tx,
            card_rx,
        })
        .bootstrap();

        // Return a sending handle
        Ok(JackHandle {
            cmd_tx,
            event_rx,
            card_tx,
        })
    }

    /// Bootstrap a smol runtime on a dedicated thread
    fn bootstrap(self: &Arc<Self>) {
        println!("Running bootstrap...");
        {
            let rt = Arc::clone(self);
            task::spawn(async move { cmd::spawn_handle(rt).await });
        }
        {
            let rt = Arc::clone(self);
            task::spawn(async move { card::spawn_handle(rt).await });
        }
        {
            let rt = Arc::clone(&self);
            task::spawn(async move { client::spawn_handle(rt).await });
        }
    }
}
