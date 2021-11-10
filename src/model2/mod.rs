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
    midi::MidiGroups,
};
use crate::{rts::jack::JackHandle, settings::Settings};
use futures_lite::future::block_on;
use smol::LocalExecutor;
use std::{collections::BTreeMap, sync::Arc, thread};

pub struct Model {
    jack_handle: JackHandle,
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
    pub fn new(jack_handle: JackHandle, settings: Arc<Settings>) -> Self {
        Self {
            jack_handle,
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
    }
}

pub fn dispatch(m: Model) {
    thread::spawn(move || {
        let local_exec = LocalExecutor::new();
        block_on(run(local_exec, m))
    });
}

async fn run<'exe>(exe: LocalExecutor<'exe>, m: Model) {
    let jack = m.jack_handle.clone();
    exe.spawn(async move {
        // ...
    });
}
