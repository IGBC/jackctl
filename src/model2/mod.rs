//! New model abstraction

pub mod card;
pub mod events;
pub mod port;

use self::card::{Card, CardId};
use self::events::{HardwareCmd, HardwareEvent, JackEvent, UiCmd, UiEvent};
use crate::rts::{hardware::HardwareHandle, jack::JackHandle};
use crate::{settings::Settings, ui::UiHandle};
use async_std::task;
use futures::FutureExt;
use std::{collections::BTreeMap, sync::Arc};

pub struct Model {
    jack_handle: JackHandle,
    ui_handle: UiHandle,
    hw_handle: HardwareHandle,
    settings: Arc<Settings>,

    /// Card data and state map
    cards: BTreeMap<CardId, Card>,
}

impl Model {
    /// Initialise a new model tree
    pub fn start(
        jack_handle: JackHandle,
        ui_handle: UiHandle,
        hw_handle: HardwareHandle,
        settings: Arc<Settings>,
    ) {
        Self {
            jack_handle,
            ui_handle,
            hw_handle,
            settings,
            cards: Default::default(),
        }
        .dispatch()
    }

    fn dispatch(self) {
        task::spawn(async move {
            run(self).await;
            println!("Model run loop shut down");
        });
    }
}

async fn run(mut m: Model) {
    let jack_handle = m.jack_handle.clone();
    let ui_handle = m.ui_handle.clone();
    let hw_handle = m.hw_handle.clone();

    loop {
        let mut jack_event_poll = Box::pin(jack_handle.next_event().fuse());
        let mut ui_event_poll = Box::pin(ui_handle.next_event().fuse());
        let mut hw_event_poll = Box::pin(hw_handle.next_event().fuse());

        futures::select! {
            ev = jack_event_poll  => match ev {
                Some(ev) => handle_jack_ev(&mut m, ev).await,
                None => return,
            },
            ev = ui_event_poll  => match ev {
                Some(ev) => handle_ui_ev(&mut m, ev).await,
                None => return,
            },
            ev = hw_event_poll  => match ev {
                Some(ev) => handle_hw_ev(&mut m, ev).await,
                None => return,
            },
        }
    }
}

/// Events from the jack runtime
async fn handle_jack_ev(m: &mut Model, ev: JackEvent) {
    println!("Selected JACK EVENT");
    use JackEvent::*;
    match ev {
        XRun => m.ui_handle.send_cmd(UiCmd::IncrementXRun).await,
        JackSettings(settings) => m.ui_handle.send_cmd(UiCmd::JackSettings(settings)).await,
        AddPort(port) => m.ui_handle.send_cmd(UiCmd::AddPort(port)).await,
        DelPort(id) => m.ui_handle.send_cmd(UiCmd::DelPort(id)).await,
        AddConnection(a, b) => m.ui_handle.send_cmd(UiCmd::AddConnection(a, b)).await,
        DelConnection(a, b) => m.ui_handle.send_cmd(UiCmd::DelConnection(a, b)).await,
    }
}

/// Events from the UI runtime
async fn handle_ui_ev(m: &mut Model, ev: UiEvent) {
    println!("Selected UI EVENT");
    use UiEvent::*;
    match ev {
        SetMuting(mute) => m.hw_handle.send_cmd(HardwareCmd::SetMixerMute(mute)).await,
        SetVolume(volume) => {
            m.hw_handle
                .send_cmd(HardwareCmd::SetMixerVolume(volume))
                .await
        }
        UpdateChannel(card, channel, volume, b) => {}
        CleanChannel(card, channel) => {}
    }
}

/// Events from the hardware runtime
async fn handle_hw_ev(_: &mut Model, ev: HardwareEvent) {
    println!("Selected HW EVENT");
    use HardwareEvent::*;
    match ev {
        NewCardFound {
            id,
            capture,
            playback,
            mixerchannels,
        } => {}
        DropCard { id } => {}
        UpdateMixerVolume {
            card,
            channel,
            volume,
        } => {}
        UpdateMixerMute {
            card,
            channel,
            mute,
        } => {}
    }
}
