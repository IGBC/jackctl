//! Types representing the JACK  connection  graph, sorted by client.
use std::cmp::PartialEq;

pub type JackPortType = u32;

/// Struct wrapping all the groups (clients) in a model for a given port type
pub struct PortGroup {
    groups: Vec<Group>,
}

/// Struct wrapping all the Ports in a group
#[derive(Clone)]
pub struct Group {
    name: String,
    ports: Vec<Port>,
}

/// An individual port in the jack server, mapped to a unique (internal) id.
#[derive(Clone, Debug, PartialEq)]
pub struct Port {
    portname: String,
    groupname: String,
    id: JackPortType,
}

/// A connection between to ports held using the JACK Server native string representation.
#[derive(Debug, PartialEq)]
pub struct Connection {
    pub input: JackPortType,
    pub output: JackPortType,
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

    pub fn remove(&mut self, index: usize) {
        self.ports.remove(index);
    }

    pub fn len(&self) -> usize {
        self.ports.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ports.is_empty()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Port> {
        self.ports.iter()
    }
}

impl Port {
    pub fn new(id: JackPortType, name: String) -> Self {
        let mut parts: Vec<&str> = name.split(':').collect();
        let groupname: String = parts.remove(0).to_owned();
        let portname = parts.join(":");

        Port {
            id,
            portname,
            groupname,
        }
    }

    pub fn name(&self) -> &str {
        &self.portname
    }

    pub fn group(&self) -> &str {
        &self.groupname
    }

    pub fn id(&self) -> JackPortType {
        self.id
    }

    pub fn fullname(&self) -> String {
        format!("{}:{}", self.groupname, self.portname)
    }
}

impl PortGroup {
    pub fn new() -> Self {
        PortGroup { groups: Vec::new() }
    }

    pub fn merge(&self, rhs: &Self) -> Self {
        let mut pg = Self::new();
        for i in self.iter() {
            pg.add_group(i.clone());
        }

        for i in rhs.iter() {
            pg.add_group(i.clone());
        }

        pg
    }

    pub fn add(&mut self, port: Port) {
        let group = port.group();
        let g: &mut Group = match self.groups.iter().position(|r| r.name() == group) {
            Some(i) => &mut self.groups[i],
            None => {
                self.groups.push(Group::new(group.to_owned()));
                self.groups.last_mut().unwrap()
            }
        };

        g.add(port);
    }

    pub fn remove_port_by_id(&mut self, id: JackPortType) -> bool {
        for (i, g) in self.groups.iter_mut().enumerate() {
            match g.iter().position(|r| r.id() == id) {
                Some(j) => {
                    g.remove(j);
                    if g.is_empty() {
                        self.groups.remove(i);
                    }

                    return true;
                }
                None => (),
            }
        }

        return false;
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

    pub fn get_port_by_id(&self, id: JackPortType) -> Option<&Port> {
        for g in self.groups.iter() {
            for p in g.iter() {
                if p.id() == id {
                    return Some(p);
                }
            }
        }
        None
    }

    pub fn get_port_name_by_id(&self, id: JackPortType) -> Option<String> {
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
