/// Port ID
pub type JackPortType = u32;

/// An individual port in the jack server, mapped to a unique (internal) id.
#[derive(Clone, Debug, PartialEq)]
pub struct Port {
    pub client_name: String,
    pub port_name: String,
    pub id: JackPortType,
    pub tt: PortType,
    pub dir: PortDirection,
    pub is_hw: bool,
}

impl Port {
    pub fn new(
        client_name: String,
        port_name: String,
        id: JackPortType,
        tt: PortType,
        dir: PortDirection,
        is_hw: bool,
    ) -> Self {
        Self {
            client_name,
            port_name,
            id,
            tt,
            dir,
            is_hw,
        }
    }
}

/// Type of port
#[derive(Clone, Debug, PartialEq)]
pub enum PortType {
    Audio,
    Midi,
    Unknown,
}

/// Direction of the port
#[derive(Clone, Debug, PartialEq)]
pub enum PortDirection {
    Input,
    Output,
}
