mod async_client;
mod card;
mod client;
mod cmd;
mod server;

use self::async_client::JackNotificationController;
use self::server::JackServer;
use crate::cb_channel::{self, ReturningReceiver, ReturningSender};
use crate::model2::events::{JackCardAction, JackCmd, JackEvent};
use crate::settings::Settings;
use async_std::{
    channel::{bounded, Receiver, Sender},
    task,
};
use jack::{AsyncClient, Client as JackClient, InternalClientID};
use std::sync::Arc;

/// An easily clonable handle to the jack runtime
#[derive(Clone)]
pub struct JackHandle {
    /// Send commands to the jack runtime
    cmd_tx: Sender<JackCmd>,
    /// Receive events from the jack runtime
    event_rx: Receiver<JackEvent>,
    /// Send card actions to jack runtime with blocking ACK
    card_tx: ReturningSender<JackCardAction, Result<InternalClientID, jack::Error>>,
}

impl JackHandle {
    /// Send a jack command to the associated task
    pub async fn send_cmd(&self, cmd: JackCmd) {
        self.cmd_tx.send(cmd).await.unwrap();
    }

    /// Wait for the next jack event
    pub async fn next_event(&self) -> Option<JackEvent> {
        println!("Polling for jack event");
        self.event_rx.recv().await.ok()
    }

    /// Send a card action and wait for the reply
    pub async fn send_card_action(
        &self,
        action: JackCardAction,
    ) -> Result<InternalClientID, jack::Error> {
        self.card_tx.send(action).await.unwrap()
    }
}

/// Jack server runtime and signalling state
pub struct JackRuntime {
    /// reference for the jack server, server will stop when dropped
    server: JackServer,
    /// Resample Quality fetched from settings on boot
    resample_q: u32,
    /// number of periods per frame, fetched from settings on boot
    n_periods: u32,
    /// Async jack client
    a_client: AsyncClient<JackNotificationController, ()>,
    /// Receive jack commands
    cmd_rx: Receiver<JackCmd>,
    /// Send events to the model layer
    event_tx: Sender<JackEvent>,
    /// Receive card commands
    card_rx: ReturningReceiver<JackCardAction, Result<InternalClientID, jack::Error>>,
}

impl JackRuntime {
    pub fn start(settings: Arc<Settings>) -> Result<JackHandle, jack::Error> {
        // start the server first
        let app_settings = settings.r().app();
        let jack_settings = &app_settings.jack;
        let server = server::JackServer::new(
            jack_settings.sample_rate,
            jack_settings.period_size,
            jack_settings.realtime,
        );

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
            server,
            a_client,
            cmd_rx,
            event_tx,
            card_rx,
            n_periods: jack_settings.n_periods,
            resample_q: jack_settings.resample_q,
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
