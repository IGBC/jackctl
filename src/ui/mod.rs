//! Jackctl GTK UI module

mod about;
mod card_query;
mod matrix;
mod mixer;
mod pages;
mod tray;
mod utils;
mod window;
mod settings;

use tray::TrayState;
use window::MainWindow;

use crate::{
    model::events::{UiCmd, UiEvent},
    settings::Settings,
};
use async_std::channel::{bounded, Receiver, Sender};
use gio::ApplicationExt;
use gtk::prelude::*;
use gtk::{Application, ApplicationBuilder};
use std::{fmt::Debug, sync::Arc};

const RESOURCES_BUNDLE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/resources.gresource"));

#[derive(Clone, Debug)]
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
            error!("Failed to send UI command!");
        }
    }
}

#[derive(Clone)]
struct EventSender(Sender<UiEvent>);

impl EventSender {
    fn send(self, e: UiEvent) {
        async_std::task::block_on(async move {
            if let Err(_) = self.0.send(e.clone()).await {
                error!("Failed to send event '{:?}'", e);
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
        let (tx, rx) = bounded(128);
        Self { tx, rx }
    }

    pub fn send(&self, t: T) {
        async_std::task::block_on(async {
            let dbg = format!("Failed to send Questionaire<{:?}>", t);
            if let Err(e) = self.tx.send(t).await {
                error!("{}", dbg);
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
        let (tx_cmd, rx_cmd) = bounded(128);
        let (tx_event, rx_event) = bounded(128);

        (
            UiRuntime { tx_event, rx_cmd },
            UiHandle { tx_cmd, rx_event },
        )
    }
}

fn on_activate(app: &Application) {
    trace!("On Activate()")
}

pub fn create_ui(settings: Arc<Settings>) -> (Arc<MainWindow>, Application, UiHandle, TrayState) {
    // Load the compiled resource bundle
    let resource_data = glib::Bytes::from(&RESOURCES_BUNDLE[..]);
    let res = gio::Resource::from_data(&resource_data).unwrap();
    gio::resources_register(&res);

    let app = ApplicationBuilder::new()
        .application_id("jackctl.segfault")
        .resource_base_path("/net/jackctl/Jackctl")
        .build();

    app.connect_activate(|app| on_activate(app));

    if gtk::init().is_err() {
        crate::log::oops(
            "Failed to start GTK, please ensure all dependancies are installed",
            1,
        );
    }

    let (rt, handle) = UiRuntime::new();
    let win = window::create(&app, settings, rt.clone());
    let tray = TrayState::new(rt, win.get_inner());
    (win, app, handle, tray)
}
