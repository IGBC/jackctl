use crate::model2::events::JackEvent;
use crate::model2::port::{PortDirection, PortType};
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
            e => PortType::Unknown(e.to_string()),
        };

        let flags = p.flags();
        let port_dir = if flags.contains(PortFlags::IS_OUTPUT) {
            PortDirection::Output
        } else {
            PortDirection::Input
        };

        Ok((port_type, port_dir))
    }

    fn sync_send(&mut self, e: JackEvent) {
        return;
        todo!()
        // this needs to get wrapped up in blocking magic because NotificationController isn't async
        // self.pipe.send(e).await.unwrap();
    }
}

impl NotificationHandler for JackNotificationController {
    fn client_registration(&mut self, _: &jack::Client, _name: &str, _is_registered: bool) {
        eprintln!("EVENT: client_registration {}, {}", _name, _is_registered);
    }

    fn port_registration(&mut self, c: &jack::Client, port_id: PortId, is_registered: bool) {
        eprintln!("EVENT: port_registration {}, {}", port_id, is_registered);
        if is_registered {
            // go get the corisponding port
            let jack_port = match c.port_by_id(port_id) {
                Some(p) => p,
                None => {
                    eprintln!("ERROR: Jack just gave us a bad PortID");
                    return;
                }
            };

            let name = match jack_port.name() {
                Ok(n) => n,
                Err(e) => {
                    eprintln!(
                        "ERROR: Jack refused to give name for port {}: {}",
                        port_id, e
                    );
                    return;
                }
            };

            // let port = JackPort::new(port_id, name.clone());

            //     let pt = match self.identify_port(&jack_port) {
            //         Ok(pt) => pt,
            //         Err(e) => {
            //             eprintln!("Error identifying port {} \"{}\": {}", port_id, name, e);
            //             return;
            //         }
            //     };

            //     let e = match pt {
            //         (PortType::Audio, PortDirection::Input) => JackEvent::AddAudioInput(port),
            //         (PortType::Audio, PortDirection::Output) => JackEvent::AddAudioOutput(port),
            //         (PortType::Midi, PortDirection::Input) => JackEvent::AddMidiInput(port),
            //         (PortType::Midi, PortDirection::Output) => JackEvent::AddMidiOutput(port),
            //         (PortType::Unknown(f), _) => {
            //             println!("Unknown port format \"{}\" for port {}", f, name);
            //             return;
            //         },
            //     };
            //     self.sync_send(e);
            // } else {
            //     self.sync_send(JackEvent::DelPort(port_id));
        }
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        _port_id: PortId,
        _old_name: &str,
        _new_name: &str,
    ) -> jack::Control {
        eprintln!(
            "EVENT: port_rename {}, {}, {}",
            _port_id, _old_name, _new_name
        );
        eprintln!("Error: port renaming unimplemented");
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: PortId,
        port_id_b: PortId,
        are_connected: bool,
    ) {
        eprintln!(
            "EVENT: ports_connected {}, {}, {}",
            port_id_a, port_id_b, are_connected
        );
        if are_connected {
            self.sync_send(JackEvent::AddConnection(port_id_b, port_id_a));
        } else {
            self.sync_send(JackEvent::DelConnection(port_id_b, port_id_a));
        }
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        eprintln!("EVENT: XRun");
        self.sync_send(JackEvent::XRun);
        jack::Control::Continue
    }
}
