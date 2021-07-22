use std::cell::RefCell;
use std::rc::Rc;

use std::collections::HashMap;

mod card;
mod port;

pub use card::*;
pub use port::*;

pub type Model = Rc<RefCell<ModelInner>>;

pub struct ModelInner {
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
    connections: Vec<Connection>,

    pub cards: HashMap<i32, Card>,
}

impl ModelInner {
    pub fn new() -> Model {
        Rc::new(RefCell::new(ModelInner {
            ixruns: 0,
            layout_dirty: true,
            cpu_percent: 0.0,
            sample_rate: 0,
            buffer_size: 0,
            latency: 0,

            audio_inputs: PortGroup::new(false),
            audio_outputs: PortGroup::new(false),
            midi_inputs: PortGroup::new(true),
            midi_outputs: PortGroup::new(true),
            connections: Vec::new(),

            cards: HashMap::new(),
        }))
    }

    pub fn increment_xruns(&mut self) {
        self.ixruns += 1;
    }

    pub fn xruns(&self) -> u32 {
        self.ixruns
    }

    pub fn reset_xruns(&mut self) {
        self.ixruns = 0;
    }

    fn map_groups(ports: &Vec<String>, is_midi: bool) -> PortGroup {
        let mut map: PortGroup = PortGroup::new(is_midi);

        for p in ports.iter() {
            map.add(p);
        }

        map
    }

    pub fn update_audio_inputs(&mut self, ports: &Vec<String>) {
        self.audio_inputs = Self::map_groups(ports, false);
        self.layout_dirty = true;
    }

    pub fn audio_inputs(&self) -> &PortGroup {
        &self.audio_inputs
    }

    pub fn update_audio_outputs(&mut self, ports: &Vec<String>) {
        self.audio_outputs = Self::map_groups(ports, false);
        self.layout_dirty = true;
    }

    pub fn audio_outputs(&self) -> &PortGroup {
        &self.audio_outputs
    }

    pub fn update_midi_inputs(&mut self, ports: &Vec<String>) {
        self.midi_inputs = Self::map_groups(ports, true);
        self.layout_dirty = true;
    }

    pub fn midi_inputs(&self) -> &PortGroup {
        &self.midi_inputs
    }

    pub fn update_midi_outputs(&mut self, ports: &Vec<String>) {
        self.midi_outputs = Self::map_groups(ports, true);
        self.layout_dirty = true;
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

    pub fn update_connections(&mut self, connections: Vec<Connection>) {
        self.connections = connections;
    }

    // call when a card is to be added to the system that has not been seen before.
    pub fn card_detected(&mut self, id: i32, name: String) {
        println!("Found Unseen Card hw:{} - {}", id, name);
        let card = Card::new(id, name);
        self.cards.insert(id, card);
    }

    pub fn connected_by_id(&self, id1: usize, id2: usize) -> bool {
        let output_name = self.outputs().get_port_name_by_id(id1);
        let input_name = self.inputs().get_port_name_by_id(id2);
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

    pub fn set_muting(&mut self, card_id: i32, channel: u32, mute: bool) {
        let card = self.cards.get_mut(&card_id);
        if card.is_some() {
            let card = card.unwrap();
            let channel = card.channels.get_mut(&channel);
            if channel.is_some() {
                channel.unwrap().switch = mute;
            }
        }
    }

    pub fn set_volume(&mut self, card_id: i32, channel: u32, volume: i64) {
        let card = self.cards.get_mut(&card_id);
        if card.is_some() {
            let card = card.unwrap();
            let channel = card.channels.get_mut(&channel);
            if channel.is_some() {
                channel.unwrap().volume = volume;
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
