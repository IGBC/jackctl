//! Jackctl's Model and Event to drive the applications's MVC pattern
use std::sync::{Arc, Mutex};

use std::collections::HashMap;

mod card;
mod port;
mod event;

pub use card::*;
pub use port::*;

pub use event::Event;

/// Wrapper around a Mutexed Copy of the Model, 
/// use this instead of the model directly to
/// easilly allow changes to the Mutex used.
pub type Model<'a> = Arc<Mutex<ModelInner<'a>>>;

/// Central Model of the MVC layout of the application,
/// you Should only ever make one of these and pass
/// mutexed references around to it.
pub struct ModelInner<'a> {
    ixruns: u32,
    pub layout_dirty: bool,

    pub cpu_percent: f32,
    pub sample_rate: usize,
    pub buffer_size: u32,
    pub latency: u64,

    audio_inputs: PortGroup,
    audio_outputs: PortGroup,
    midi_inputs: PortGroup,
    midi_outputs: PortGroup,
    connections: Vec<Connection<'a>>,

    pub cards: HashMap<i32, Card>,
}

impl<'a> ModelInner<'a> {
   /// Returns a new model, in default state. Don't assume
   /// anything is configured or initialised in this constructor.   
    pub fn new() -> Model<'a> {
        Arc::new(Mutex::new(ModelInner {
            ixruns: 0,
            layout_dirty: true,
            cpu_percent: 0.0,
            sample_rate: 0,
            buffer_size: 0,
            latency: 0,

            audio_inputs: PortGroup::new(),
            audio_outputs: PortGroup::new(),
            midi_inputs: PortGroup::new(),
            midi_outputs: PortGroup::new(),
            connections: Vec::new(),

            cards: HashMap::new(),
        }))
    }

    pub fn update(&mut self, evt: Event<'a>) {
        match evt {
            Event::XRun => self.increment_xruns(),
            Event::ResetXruns => self.reset_xruns(),
            
            Event::AddCard(id, name) => self.card_detected(id, name),
            Event::SetMuting(id, ch, m) => self.set_muting(id, ch, m),
            Event::SetVolume(id, ch, v) => self.set_volume(id, ch, v),
            
            Event::AddAudioInput(i) => self.audio_inputs.add(i),
            Event::AddAudioOutput(o) => self.audio_outputs.add(o),
            Event::AddMidiInput(i) => self.midi_inputs.add(i),
            Event::AddMidiOutput(o) => self.midi_outputs.add(o),
            
            Event::DelPort(id) => self.del_port(id),

            Event::SyncConnections(c) => self.update_connections(c),

            _ => eprintln!("Unimplemented event")

        }

        self.layout_dirty = true;
    }

    fn increment_xruns(&mut self) {
        self.ixruns += 1;
    }

    pub fn xruns(&self) -> u32 {
        self.ixruns
    }

    fn reset_xruns(&mut self) {
        self.ixruns = 0;
    }

    fn del_port(&mut self, id: JackPortType) {
        if self.audio_inputs.remove_port_by_id(id) { return };
        if self.audio_outputs.remove_port_by_id(id) { return };
        if self.midi_inputs.remove_port_by_id(id) { return };
        if self.midi_outputs.remove_port_by_id(id) { return };
    }

    pub fn audio_inputs(&self) -> &PortGroup {
        &self.audio_inputs
    }

    pub fn audio_outputs(&self) -> &PortGroup {
        &self.audio_outputs
    }

    pub fn midi_inputs(&self) -> &PortGroup {
        &self.midi_inputs
    }

    pub fn midi_outputs(&self) -> &PortGroup {
        &self.midi_outputs
    }

    pub fn inputs(&self) -> PortGroup {
        self.audio_inputs.merge(&self.midi_inputs)
    }

    pub fn outputs(&self) -> PortGroup {
        self.audio_outputs.merge(&self.midi_outputs)
    }

    fn update_connections(&mut self, connections: Vec<Connection<'a>>) {
        self.connections = connections;
    }

    // call when a card is to be added to the system that has not been seen before.
    fn card_detected(&mut self, id: i32, name: String) {
        println!("Found Unseen Card hw:{} - {}", id, name);
        let card = Card::new(id, name);
        self.cards.insert(id, card);
    }

    // TODO: make this work with just ID's
    pub fn connected_by_id(&self, id1: JackPortType, id2: JackPortType) -> bool {
        let outputs = self.outputs();
        let inputs = self.inputs();
        let output_name = outputs.get_port_by_id(id1);
        let input_name = inputs.get_port_by_id(id2);
        if output_name.is_none() || input_name.is_none() {
            return false;
        }
        
        let output_name = output_name.unwrap();
        let input_name = input_name.unwrap();
        for c in self.connections.iter() {
            if (c.input == input_name) && (c.output == output_name) {
                return true;
            }
        }
        false
    }

    fn set_muting(&mut self, card_id: i32, channel: u32, mute: bool) {
        let card = self.cards.get_mut(&card_id);
        if card.is_some() {
            let card = card.unwrap();
            let channel = card.channels.get_mut(&channel);
            if channel.is_some() {
                let mut channel = channel.unwrap();
                channel.switch = mute;
                channel.dirty = true;
            }
        }
    }

    fn set_volume(&mut self, card_id: i32, channel: u32, volume: i64) {
        let card = self.cards.get_mut(&card_id);
        if card.is_some() {
            let card = card.unwrap();
            let channel = card.channels.get_mut(&channel);
            if channel.is_some() {
                let mut channel = channel.unwrap();
                channel.volume = volume;
                channel.dirty = true;
            }
        }
    }

    pub fn get_muting(&self, card_id: i32, channel: u32) -> bool {
        match self.cards.get(&card_id) {
            Some(card) => match card.channels.get(&channel) {
                Some(channel) => channel.switch,
                None => false,
            },
            None => false,
        }
    }

    pub fn get_volume(&self, card_id: i32, channel: u32) -> i64 {
        match self.cards.get(&card_id) {
            Some(card) => match card.channels.get(&channel) {
                Some(channel) => channel.volume,
                None => 0,
            },
            None => 0,
        }
    }
}
