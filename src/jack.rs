//! Jackctl's connection to the JACK server.

use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

use gtk::prelude::*;

use jack::Client as JackClient;
use jack::{PortFlags, NotificationHandler, PortId, AsyncClient};

use crate::model::{Connection, Model, Event, JackPortType};

pub struct JackNotificationController {
    model: Model<'static>,
}

/// Controller that manages the connection to the JACK server.
pub struct JackController {
    model: Model<'static>,
    interface: AsyncClient<JackNotificationController, ()>,
    // async_interface: jack::AsyncClient<JackNotificationController, ()>,
    // old_audio_inputs: Vec<String>,
    // old_audio_outputs: Vec<String>,
    // old_midi_inputs: Vec<String>,
    // old_midi_outputs: Vec<String>,
}

impl JackController {
    /// Creates new connection to the JACK server.
    /// This function will loop untill the connection succeeds.
    pub fn new(model: Model<'static>) -> Rc<RefCell<Self>> {
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
            model,
        };
        
        let interface = client.activate_async(async_controller, ()).unwrap();
        
        let this = Rc::new(RefCell::new(Self {
            model,
            interface,
        }));
        
        this.borrow_mut().update_model();
        let this_clone = this.clone();
        glib::timeout_add_local(200, move || {
            this_clone.borrow_mut().update_model();
            Continue(true)
        });

        this
    }

    /// Connect two jack ports together on the server.
    pub fn connect_ports(&self, portid1: JackPortType, portid2: JackPortType, connect: bool) -> bool {
        let model = self.model.borrow();
        let input = model.inputs().get_port_name_by_id(portid2);
        let output = model.outputs().get_port_name_by_id(portid1);
        if input.is_none() || output.is_none() {
            println!("Cannot Connect: {:?} to {:?}", output, input);
            !connect
        } else {
            let input = input.unwrap();
            let output = output.unwrap();
            let result = if connect {
                self.interface.connect_ports_by_name(&output, &input)
            } else {
                self.interface.disconnect_ports_by_name(&output, &input)
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

    /// Interogates the jack server for changes, and submits them to the [Model](crate::model::ModelInner)
    pub fn update_model(&mut self) {
        let mut model = self.model.borrow_mut();
        model.cpu_percent = self.interface.cpu_load();
        model.sample_rate = self.interface.sample_rate();
        let frames = self.interface.buffer_size();
        model.buffer_size = frames.into();
        model.latency = (model.buffer_size) as u64 / (model.sample_rate as u64 / 1000u64) as u64;

        let inputs = self.interface.ports(None, None, PortFlags::IS_INPUT);
        let (ap, mp) = self.split_midi_ports(inputs.clone());

        let ap = self.filter_ports(ap);

        //Check audio ports changed
        if ap != self.old_audio_inputs {
            model.update(Event::SyncAudioInputs(ap.clone()));
            self.old_audio_inputs = ap;
        }

        // check midi ports changed
        if mp != self.old_midi_inputs {
            model.update(Event::SyncMidiInputs(mp.clone()));
            self.old_midi_inputs = mp;
        }

        let outputs = self.interface.ports(None, None, PortFlags::IS_OUTPUT);
        let (ap, mp) = self.split_midi_ports(outputs.clone());

        let ap = self.filter_ports(ap);

        // Check audio ports changed
        if ap != self.old_audio_outputs {
            model.update(Event::SyncAudioOutputs(ap.clone()));
            self.old_audio_outputs = ap;
        }

        // Check midi ports changed
        if mp != self.old_midi_outputs {
            model.update(Event::SyncMidiOutputs(mp.clone()));
            self.old_midi_outputs = mp;
        }

        let mut connections = Vec::new();
        for o in outputs.iter() {
            let output = self
                .interface
                .port_by_name(&o)
                .expect("should always exist");
            for i in inputs.iter() {
                match output.is_connected_to(&i) {
                    Ok(true) => {
                        let c = Connection {
                            input: i,
                            output: o,
                        };
                        connections.push(c);
                    }
                    _ => (),
                }
            }
        }

        model.update(Event::SyncConnections(connections));
    }

    fn split_midi_ports(&self, ports: Vec<String>) -> (Vec<String>, Vec<String>) {
        let mut audio_ports: Vec<String> = Vec::new();
        let mut midi_ports: Vec<String> = Vec::new();
        for name in ports.iter() {
            let port = self.interface.port_by_name(name).unwrap();
            match port.port_type() {
                Ok(t) => match t.as_str() {
                    "32 bit float mono audio" => audio_ports.push(name.to_owned()),
                    "8 bit raw midi" => midi_ports.push(name.to_owned()),
                    e => println!("Unknown port format \"{}\" for port {}", e, name),
                },
                Err(e) => println!("Could not open port {}: {}", name, e.to_string()),
            }
        }
        (audio_ports, midi_ports)
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
}


impl NotificationHandler for JackNotificationController {
    fn client_registration(&mut self, _: &jack::Client, _name: &str, _is_registered: bool) {
        eprintln!("EVENT: client_registration {}, {}", _name, _is_registered);
    }

    fn port_registration(&mut self, _: &jack::Client, _port_id: PortId, _is_registered: bool) {
        eprintln!("EVENT: port_registration {}, {}", _port_id, _is_registered);
    }

    fn port_rename(&mut self, _: &jack::Client, _port_id: PortId, _old_name: &str, _new_name: &str) -> jack::Control {
        eprintln!("EVENT: port_rename {}, {}, {}", _port_id, _old_name, _new_name);
        jack::Control::Continue
    }

    fn ports_connected(&mut self, _: &jack::Client, _port_id_a: PortId, _port_id_b: PortId, _are_connected: bool) {
        eprintln!("EVENT: ports_connected {}, {}, {}", _port_id_a, _port_id_b, _are_connected);
    }
}
