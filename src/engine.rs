use std::rc::Rc;
use std::cell::RefCell;

use gtk::prelude::*;

use jack::Client as JackClient;
use jack::PortFlags;

use crate::model::{Model, Connection};

pub struct Controller {
    model: Model,
    interface: JackClient,
    old_inputs: Vec<String>,
    old_outputs: Vec<String>,
}

impl Controller {
    pub fn new(model: Model) -> Rc<RefCell<Self>> {
        let this = Rc::new(RefCell::new(Controller {
            model: model,
            old_inputs: Vec::new(),
            old_outputs: Vec::new(),
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
        if inputs != self.old_inputs {
            model.update_inputs(&inputs);
            self.old_inputs = inputs.clone();
        }
        
        let outputs = self.interface.ports(None, None, PortFlags::IS_OUTPUT);
        if outputs != self.old_outputs {
            model.update_outputs(&outputs);
            self.old_outputs = outputs.clone();
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


        // println!("{:?}", self.get_all_ports());
    }

    // fn split_midi_ports(ports: Vec<String>) -> (Vec<String>, Vec<String>) {
    //     for name in ports.iter() {
    //         port = self.interface.port_by_name(name);
    //     }
    // }

    // fn get_all_ports(&self) -> Vec<jack::Port<jack::Unowned>> {
    //     let mut i = 1;
    //     let mut cont = true;
    //     let mut ports: Vec<jack::Port<jack::Unowned>> = Vec::new();
    //     while cont {
    //         match self.interface.port_by_id(i){
    //             Some(port) =>  {
    //                 println!("{:?}",port);
    //                 if &port.name().unwrap_or("".to_owned())  == "" {
    //                     cont = false;
    //                 } else {
    //                     ports.push(port);
    //                     i += 1;
    //                 }
    //             },
    //             None => cont = false,
    //         }
    //     }
    //     ports
    // }
}