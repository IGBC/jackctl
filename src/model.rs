use std::cell::RefCell;
use std::rc::Rc;

pub type Model = Rc<RefCell<ModelInner>>;

#[derive(Default)]
pub struct PortGroup {
    groups: Vec<Group>,
}

#[derive(Default)]
pub struct Group {
    name: String,
    ports: Vec<Port>,
}

pub struct Port {
    portname: String,
    id: usize,
}

#[derive(Debug)]
pub struct Connection {
    pub input: String,
    pub output: String,
}

#[derive(Default)]
pub struct ModelInner {
    ixruns: u32,
    pub layout_dirty: bool,
    
    pub cpu_percent: f32,
    pub sample_rate: usize,
    pub buffer_size: u32,
    pub latency:     u64,

    inputs: PortGroup,
    outputs: PortGroup,
    connections: Vec<Connection>,
}

impl ModelInner {
    pub fn new() -> Model {
        Rc::new(RefCell::new(ModelInner::default()))
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

    fn map_groups(ports: &Vec<String>) -> PortGroup {
        let mut map: PortGroup = PortGroup::default();

        for p in ports.iter() {
            map.add(p);
        }

        map
    }

    pub fn update_inputs(&mut self, ports: &Vec<String>) {
        self.inputs = Self::map_groups(ports);
        self.layout_dirty = true;
    }

    pub fn inputs(&self) -> &PortGroup {
        &self.inputs
    }

    pub fn update_outputs(&mut self, ports: &Vec<String>) {
        self.outputs = Self::map_groups(ports);
        self.layout_dirty = true;
    }

    pub fn outputs(&self) -> &PortGroup {
        &self.outputs
    }

    pub fn update_connections(&mut self, connections: Vec<Connection>) {
        self.connections = connections;
    }

    pub fn connected_by_id(&self, id1: usize, id2: usize) -> bool {
        let output_name = self.outputs.get_port_name_by_id(id1);
        let input_name = self.inputs.get_port_name_by_id(id2);
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
    pub fn add(&mut self, name: &str) {
        let mut parts: Vec<&str> = name.split(':').collect();
        let group: String = parts.remove(0).to_owned();
        let portname = parts.join(":");

        let port = Port {
            portname,
            id: self.len(),
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