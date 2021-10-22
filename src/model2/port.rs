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

/// Type of port
pub enum PortType {
    Audio,
    Midi,
    Unknown(String),
}

/// Direction of the port
pub enum PortDirection {
    Input,
    Output,
}
