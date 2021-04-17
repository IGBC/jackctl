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

pub type Port = String;

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
            let mut parts: Vec<&str> = p.split(':').collect();
            let group: &str = parts.remove(0);
            let name = parts.join(":");
            
            map.add(group.to_owned(), name);
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


impl PortGroup {
    pub fn add(&mut self, group: String, port: Port) {
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
}