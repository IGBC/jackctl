//! Jackctl GTK UI module

mod about;
mod card_query;
mod matrix;
mod mixer;
mod pages;
mod utils;
mod window;

use window::MainWindow;

use crate::{
    model::events::{UiCmd, UiEvent},
    settings::Settings,
};
use async_std::channel::{bounded, Receiver, Sender};
use gtk::{Application, Builder};
use std::{fmt::Debug, sync::Arc};

const GLADEFILE: &str = include_str!("../jackctl.glade");

#[derive(Clone)]
pub struct UiHandle {
    tx_cmd: Sender<UiCmd>,
    rx_event: Receiver<UiEvent>,
}

impl UiHandle {
    pub async fn next_event(&self) -> Option<UiEvent> {
        self.rx_event.recv().await.ok()
    }
    pub async fn send_cmd(&self, cmd: UiCmd) {
        if let Err(_) = self.tx_cmd.send(cmd).await {
            println!("Failed to send UI command!");
        }
    }
}

#[derive(Clone)]
struct EventSender(Sender<UiEvent>);

impl EventSender {
    fn send(self, e: UiEvent) {
        async_std::task::block_on(async move {
            if let Err(_) = self.0.send(e.clone()).await {
                println!("Failed to send event '{:?}'", e);
            }
        });
    }
}

/// A channel to convey information about questions to the user
#[derive(Clone)]
struct Questionaire<T> {
    tx: Sender<T>,
    rx: Receiver<T>,
}

impl<T: Debug> Questionaire<T> {
    pub fn new() -> Self {
        let (tx, rx) = bounded(12);
        Self { tx, rx }
    }

    pub fn send(&self, t: T) {
        async_std::task::block_on(async {
            let dbg = format!("Failed to send Questionaire<{:?}>", t);
            if let Err(e) = self.tx.send(t).await {
                println!("{}", dbg);
            }
        });
    }

    pub fn try_recv(&self) -> Option<T> {
        self.rx.try_recv().ok()
    }
}

#[derive(Clone)]
struct UiRuntime {
    tx_event: Sender<UiEvent>,
    rx_cmd: Receiver<UiCmd>,
}

impl UiRuntime {
    fn sender(&self) -> EventSender {
        EventSender(self.tx_event.clone())
    }

    /// Try to read events from the channel, up to a maximum number
    ///
    /// Returns `None` if the channel was empty or closed
    #[inline]
    fn get_cmds(&self, max: u8) -> Option<Vec<UiCmd>> {
        let mut buf = vec![];
        for _ in 0..max {
            match self.rx_cmd.try_recv() {
                Ok(ev) => buf.push(ev),
                Err(_) if buf.is_empty() => return None,
                Err(_) => break,
            }
        }

        Some(buf)
    }

    fn new() -> (Self, UiHandle) {
        let (tx_cmd, rx_cmd) = bounded(8);
        let (tx_event, rx_event) = bounded(8);

        (
            UiRuntime { tx_event, rx_cmd },
            UiHandle { tx_cmd, rx_event },
        )
    }
}

pub fn create_ui(settings: Arc<Settings>) -> (Arc<MainWindow>, Application, UiHandle) {
    if gtk::init().is_err() {
        println!("Failed to start GTK, please ensure all dependancies are installed");
    }

    let (rt, handle) = UiRuntime::new();
    let builder = Builder::from_string(GLADEFILE);
    let (win, app) = window::create(settings, builder, rt);
    (win, app, handle)
}
