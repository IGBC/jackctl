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

impl Group {
    pub fn new(name: String) -> Self {
        Group {
            name,
            ports: Vec::new(),
        }
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
    pub fn name(&self) -> &str {
        &self.portname
    }
    pub fn id(&self) -> usize {
        self.id
    }
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

        let g: &mut Group = match self.groups.iter().position(|r| r.name() == &group) {
            Some(i) => &mut self.groups[i],
            None => {
                self.groups.push(Group::new(group));
                self.groups.last_mut().unwrap()
            }
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
        self.groups.iter().map(|p| p.len()).sum()
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
                    return Some([g.name(), p.name()].join(":"));
                }
            }
        }
        None
    }
}