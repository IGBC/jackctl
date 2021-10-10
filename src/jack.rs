//! Jackctl's connection to the JACK server.

use crate::model::{Event, JackPortType, Model, Port};
use gtk::prelude::*;
use jack::Client as JackClient;
use jack::Error as JackError;
use jack::InternalClientID;
use jack::Port as JackPort;
use jack::{AsyncClient, NotificationHandler, PortFlags, PortId, Unowned};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
enum PortType {
    Audio,
    Midi,
    Unknown(String),
}

enum PortDirection {
    Input,
    Output,
}

pub struct JackNotificationController {
    model: Model,
}

/// Controller that manages the connection to the JACK server.
pub struct JackController {
    model: Model,
    interface: AsyncClient<JackNotificationController, ()>,
    cards: HashMap<String, InternalClientID>,
}

impl JackController {
    /// Creates new connection to the JACK server.
    /// This function will loop untill the connection succeeds.
    pub fn new(model: Model) -> Rc<RefCell<Self>> {
        let client = loop {
            match JackClient::new("jackctl", jack::ClientOptions::NO_START_SERVER) {
                Ok(i) => {
                    break i.0;
                }
                Err(e) => {
                    println!("{:?}", e);
                    thread::sleep(Duration::from_secs(2));
                }
            }
        };

        let async_controller = JackNotificationController {
            model: model.clone(),
        };

        let interface = client.activate_async(async_controller, ()).unwrap();

        let this = Rc::new(RefCell::new(Self {
            model,
            interface,
            cards: HashMap::new(),
        }));

        this.borrow_mut().interval_update();
        let this_clone = this.clone();
        glib::timeout_add_local(200, move || {
            this_clone.borrow_mut().interval_update();
            Continue(true)
        });

        this
    }

    /// Connect two jack ports together on the server.
    pub fn connect_ports(
        &self,
        portid1: JackPortType,
        portid2: JackPortType,
        connect: bool,
    ) -> bool {
        let model = self.model.lock().unwrap();
        let input = model.inputs().get_port_name_by_id(portid2);
        let output = model.outputs().get_port_name_by_id(portid1);
        if input.is_none() || output.is_none() {
            println!("Cannot Connect: {:?} to {:?}", output, input);
            !connect
        } else {
            let input = input.unwrap();
            let output = output.unwrap();
            let result = if connect {
                self.interface
                    .as_client()
                    .connect_ports_by_name(&output, &input)
            } else {
                self.interface
                    .as_client()
                    .disconnect_ports_by_name(&output, &input)
            };
            match result {
                Ok(()) => connect,
                Err(e) => {
                    println!("Connection Error: {}", e);
                    !connect
                }
            }
        }
    }

    /// Interogates the jack server for changes, that cannot be streamed as events, and submits
    /// them to the [Model](crate::model::ModelInner)
    pub fn interval_update(&mut self) {
        let mut model = self.model.lock().unwrap();
        let interface = self.interface.as_client();
        model.cpu_percent = interface.cpu_load();
        model.sample_rate = interface.sample_rate();
        let frames = interface.buffer_size();
        model.buffer_size = frames.into();
        model.latency = (model.buffer_size) as u64 / (model.sample_rate as u64 / 1000u64) as u64;
    }

    fn filter_ports(&self, ports: Vec<String>) -> Vec<String> {
        let mut hard_ports = Vec::new();
        let mut soft_ports = Vec::new();

        for i in ports.iter() {
            let g: &str = i.split(':').collect::<Vec<&str>>()[0];
            if g.ends_with(" - Outputs") || g.ends_with(" - Inputs") {
                hard_ports.push(i.clone());
            } else {
                if g != "system" {
                    soft_ports.push(i.clone());
                }
            }
        }

        let mut out = hard_ports;
        out.append(&mut soft_ports);
        out
    }

    fn launch_card(
        &mut self,
        id: &str,
        name: &str,
        in_ports: u32,
        out_ports: u32,
        nperiods: u32,
        quality: u32,
    ) -> Result<(), jack::Error> {
        let client = self.interface.as_client();
        let rate = client.sample_rate();
        let psize = client.buffer_size();
        let args = format!(
            "-d {} -r {} -p {} -n {} -i {} -o {} -q {}",
            id, rate, psize, nperiods, in_ports, out_ports, quality
        );
        let result = client.load_internal_client(name, "audioadapter", &args)?;
        self.cards.insert(id.to_owned(), result);
        Ok(())
    }

    fn recover_card(&mut self, id: &str) -> bool {
        let key = self.cards.get(id);
        match key {
            Some(id) => {
                let result = self.interface.as_client().unload_internal_client(*id);
                result.is_ok()
            }
            None => true,
        }
    }
}

impl JackNotificationController {
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

            let port = Port::new(port_id, name.clone());

            let mut model = self.model.lock().unwrap();

            let pt = match self.identify_port(&jack_port) {
                Ok(pt) => pt,
                Err(e) => {
                    eprintln!("Error identifying port {} \"{}\": {}", port_id, name, e);
                    return;
                }
            };

            match pt {
                (PortType::Audio, PortDirection::Input) => model.update(Event::AddAudioInput(port)),
                (PortType::Audio, PortDirection::Output) => {
                    model.update(Event::AddAudioOutput(port))
                }
                (PortType::Midi, PortDirection::Input) => model.update(Event::AddMidiInput(port)),
                (PortType::Midi, PortDirection::Output) => model.update(Event::AddMidiOutput(port)),
                (PortType::Unknown(f), _) => {
                    println!("Unknown port format \"{}\" for port {}", f, name);
                    return;
                }
            }
        } else {
            let mut model = self.model.lock().unwrap();
            model.update(Event::DelPort(port_id));
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
        let mut model = self.model.lock().unwrap();
        if are_connected {
            model.update(Event::AddConnection(port_id_b, port_id_a));
        } else {
            model.update(Event::DelConnection(port_id_b, port_id_a));
        }
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        eprintln!("EVENT: XRun");
        let mut model = self.model.lock().unwrap();
        model.update(Event::XRun);
        jack::Control::Continue
    }
}
