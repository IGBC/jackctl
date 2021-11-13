//! New model abstraction

pub mod audio;
pub mod card;
pub mod con;
pub mod events;
pub mod midi;
pub mod port;

use self::{
    audio::AudioGroups,
    card::{Card, CardId},
    con::Connections,
    events::{JackEvent, UiEvent},
    midi::MidiGroups,
};
use crate::{
    rts::{
        hardware::{HardwareEvent, HardwareHandle},
        jack::JackHandle,
    },
    settings::Settings,
    ui::UiHandle,
};
use async_std::task;
use futures::FutureExt;
use std::{collections::BTreeMap, sync::Arc};

pub struct Model {
    jack_handle: JackHandle,
    ui_handle: UiHandle,
    hw_handle: HardwareHandle,
    settings: Arc<Settings>,

    x_runs: u32,
    cpu_percent: f32,
    sample_rate: u64,
    buffer_size: u64,
    latency: u64,

    /// Audio I/O data
    audio_groups: AudioGroups,
    /// Midi I/O data
    midi_groups: MidiGroups,
    /// Actively patched connections
    connections: Connections,
    /// Card data map
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
            x_runs: 0,
            cpu_percent: 0.0,
            sample_rate: 0,
            buffer_size: 0,
            latency: 0,
            audio_groups: Default::default(),
            midi_groups: Default::default(),
            connections: Default::default(),
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

async fn handle_jack_ev(_: &mut Model, ev: JackEvent) {}
async fn handle_ui_ev(_: &mut Model, ev: UiEvent) {}
async fn handle_hw_ev(_: &mut Model, ev: HardwareEvent) {}
