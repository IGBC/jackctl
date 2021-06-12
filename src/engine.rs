use std::rc::Rc;
use std::cell::RefCell;

use gtk::prelude::*;

use jack::Client as JackClient;
use jack::PortFlags;

use crate::model::{Model, Connection};

pub struct Controller {
    model: Model,
    interface: JackClient,
    old_audio_inputs: Vec<String>,
    old_audio_outputs: Vec<String>,
    old_midi_inputs: Vec<String>,
    old_midi_outputs: Vec<String>,
}

impl Controller {
    pub fn new(model: Model) -> Rc<RefCell<Self>> {
        let this = Rc::new(RefCell::new(Controller {
            model: model,
            old_audio_inputs: Vec::new(),
            old_audio_outputs: Vec::new(),
            old_midi_inputs: Vec::new(),
            old_midi_outputs: Vec::new(),
            interface: JackClient::new("jackctl", jack::ClientOptions::NO_START_SERVER).unwrap().0,
        }));
        this.borrow_mut().update_ui();
        let this_clone = this.clone();
        glib::timeout_add_local(200, move || {this_clone.borrow_mut().update_ui(); Continue(true)});

        this
    }

    pub fn connect_ports(&self, portid1: usize, portid2: usize, connect: bool) -> bool {
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

    pub fn update_ui(&mut self) {
        let mut model = self.model.borrow_mut();
        model.cpu_percent = self.interface.cpu_load();
        model.sample_rate = self.interface.sample_rate();
        let frames = self.interface.buffer_size();
        model.buffer_size = frames.into();
        model.latency = self.interface.frames_to_time(1) / 1000;

        let inputs = self.interface.ports(None, None, PortFlags::IS_INPUT);
        let (ap, mp) = self.split_midi_ports(inputs.clone());
        //Check audio ports changed
        if ap != self.old_audio_inputs {
            model.update_audio_inputs(&ap);
            self.old_audio_inputs = ap;
        }
        
        // check midi ports changed 
        if mp != self.old_midi_inputs {
            model.update_midi_inputs(&mp);
            self.old_midi_inputs = mp;
        }
        
        let outputs = self.interface.ports(None, None, PortFlags::IS_OUTPUT);
        let (ap, mp) = self.split_midi_ports(outputs.clone());
        
        // Check audio ports changed
        if ap != self.old_audio_outputs {
            model.update_audio_outputs(&ap);
            self.old_audio_outputs = ap;
        }

        // Check midi ports changed
        if mp != self.old_midi_outputs {
            model.update_midi_outputs(&mp);
            self.old_midi_outputs = mp;
        }

        let mut connections = Vec::new();
        for o in outputs.iter() {
            let output = self.interface.port_by_name(&o).expect("should always exist");
            for i in inputs.iter() {
                match output.is_connected_to(&i) {
                    Ok(true) => {
                        let c = Connection {
                            input: i.to_owned(),
                            output: o.to_owned(),
                        };
                        connections.push(c);
                    },
                    _ => (),
                }
            }
        }

        model.update_connections(connections);
    }

    fn split_midi_ports(&self, ports: Vec<String>) -> (Vec<String>, Vec<String>) {
        let mut audio_ports:Vec<String> = Vec::new();
        let mut midi_ports:Vec<String> = Vec::new();
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
}