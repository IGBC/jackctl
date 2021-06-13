use std::cell::RefCell;
use std::rc::Rc;

use crate::mixer::MixerModel;

pub type Model = Rc<RefCell<ModelInner>>;

pub struct PortGroup {
    is_midi: bool,
    groups: Vec<Group>,
}

#[derive(Clone)]
pub struct Group {
    name: String,
    ports: Vec<Port>,
}

#[derive(Clone)]
pub struct Port {
    portname: String,
    id: usize,
}

#[derive(Debug)]
pub struct Connection {
    pub input: String,
    pub output: String,
}

pub struct ModelInner {
    ixruns: u32,
    pub layout_dirty: bool,
    
    pub cpu_percent: f32,
    pub sample_rate: usize,
    pub buffer_size: u32,
    pub latency:     u64,

    audio_inputs: PortGroup,
    audio_outputs: PortGroup,
    midi_inputs: PortGroup,
    midi_outputs: PortGroup,
    connections: Vec<Connection>,

    mixer: MixerModel,
}

impl ModelInner {
    pub fn new() -> Model {
        let mixer = MixerModel::new();
        println!("MixerModel: {:?}", mixer);

        Rc::new(RefCell::new(ModelInner{
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

            mixer,
        }))
    }

    pub fn increment_xruns(&mut self) {
        self.ixruns += 1;
        println!("Xruns: {}", self.ixruns);
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
}

impl Group {
    pub fn new(name: String) -> Self {
        Group { name, ports: Vec::new() }
    }

    pub fn add(&mut self, port: Port) {
        self.ports.push(port)
    }

    pub fn len(&self) -> usize {
        self.ports.len()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Port> {
        self.ports.iter()
    }
}

impl Port {
    pub fn name(&self) -> &str { &self.portname }
    pub fn id(&self) -> usize { self.id }
}


impl PortGroup {
    pub fn new(is_midi: bool) -> Self {
        PortGroup {
            is_midi,
            groups: Vec::new(),
        }
    }

    pub fn merge(&self, rhs: &Self) -> Self {
        let mut pg = Self::new(false);
        for i in self.iter() {
            pg.add_group(i.clone());
        }

        for i in rhs.iter() {
            pg.add_group(i.clone());
        }

        pg
    }

    pub fn add(&mut self, name: &str) {
        let mut parts: Vec<&str> = name.split(':').collect();
        let group: String = parts.remove(0).to_owned();
        let portname = parts.join(":");

        let port = if self.is_midi {
            Port {
                portname,
                id: self.len() + 1000,
            }
        } else {
            Port {
                portname,
                id: self.len(),
            }
        };

        let g: &mut Group = match self.groups.iter().position( |r| r.name() == &group) {
            Some(i) => &mut self.groups[i],
            None    => {
                self.groups.push(Group::new(group));
                self.groups.last_mut().unwrap()
            },
        };

        g.add(port);
    }

    fn add_group(&mut self, group: Group) {
        self.groups.push(group);
    }

    pub fn no_groups(&self) -> usize {
        self.groups.len()
    }

    pub fn len(&self) -> usize {
        self.groups.iter().map(|p|{p.len()}).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Group> {
        self.groups.iter()
    }

    pub fn get_port_name_by_id(&self, id: usize) -> Option<String> {
        for g in self.groups.iter() {
            for p in g.iter() {
                if p.id() == id {
                    return Some([g.name(),p.name()].join(":"));
                }
            }
        }
        None
    }
}