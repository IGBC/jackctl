use crate::model::events::JackEvent;
use crate::model::port::{Port, PortDirection, PortType};
use async_std::channel::Sender;
use jack::Error as JackError;
use jack::{NotificationHandler, Port as JackPort, PortFlags, PortId, Unowned};

pub struct JackNotificationController {
    pipe: Sender<JackEvent>,
}

impl JackNotificationController {
    pub fn new(pipe: Sender<JackEvent>) -> Self {
        Self { pipe }
    }

    fn identify_port(&self, p: &JackPort<Unowned>) -> Result<(PortType, PortDirection), JackError> {
        let port_type = match p.port_type()?.as_str() {
            "32 bit float mono audio" => PortType::Audio,
            "8 bit raw midi" => PortType::Midi,
            e => {
                warn!("Unknown port type: {}", e);
                PortType::Unknown
            }
        };

        trace!("=== {:?} ===", p.name());
        let flags = p.flags();
        let port_dir = if flags.contains(PortFlags::IS_OUTPUT) {
            PortDirection::Output
        } else {
            PortDirection::Input
        };

        Ok((port_type, port_dir))
    }

    fn sync_send(&mut self, e: JackEvent) {
        async_std::task::block_on(async {
            match self.pipe.send(e).await {
                Ok(_) => (),
                Err(e) => {
                    crate::log::oops(format!("FATAL ERROR: JACK Async Event tx - {}", e), 1);
                }
            }
        });
    }
}

impl NotificationHandler for JackNotificationController {
    fn client_registration(&mut self, _: &jack::Client, _name: &str, _is_registered: bool) {
        trace!("EVENT: client_registration {}, {}", _name, _is_registered);
    }

    fn port_registration(&mut self, c: &jack::Client, port_id: PortId, is_registered: bool) {
        trace!("EVENT: port_registration {}, {}", port_id, is_registered);
        if is_registered {
            // go get the corisponding port
            let jack_port = match c.port_by_id(port_id) {
                Some(p) => p,
                None => {
                    error!("Jack just gave us a bad PortID");
                    return;
                }
            };

            let name = match jack_port.name() {
                Ok(n) => n,
                Err(e) => {
                    error!("Jack refused to give name for port {}: {}", port_id, e);
                    return;
                }
            };

            let names: Vec<&str> = name.split(":").collect();
            let client_name = names[0];
            let port_name = names[1];

            let (tt, dir) = match self.identify_port(&jack_port) {
                Ok(pt) => pt,
                Err(e) => {
                    error!("Error identifying port {} \"{}\": {}", port_id, name, e);
                    return;
                }
            };

            // TODO: is this hardware?
            let port = Port::new(
                client_name.to_owned(),
                port_name.to_owned(),
                port_id,
                tt,
                dir,
                false,
            );
            self.sync_send(JackEvent::AddPort(port));
        } else {
            self.sync_send(JackEvent::DelPort(port_id));
        }
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        _port_id: PortId,
        _old_name: &str,
        _new_name: &str,
    ) -> jack::Control {
        trace!(
            "EVENT: port_rename {}, {}, {}",
            _port_id,
            _old_name,
            _new_name
        );
        error!("Port renaming unimplemented");
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: PortId,
        port_id_b: PortId,
        are_connected: bool,
    ) {
        trace!(
            "EVENT: ports_connected {}, {}, {}",
            port_id_a,
            port_id_b,
            are_connected
        );
        if are_connected {
            self.sync_send(JackEvent::AddConnection(port_id_b, port_id_a));
        } else {
            self.sync_send(JackEvent::DelConnection(port_id_b, port_id_a));
        }
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        trace!("EVENT: XRun");
        self.sync_send(JackEvent::XRun);
        jack::Control::Continue
    }
}
