use psutil::process;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;

use gtk::prelude::*;

use crate::model::Model;
use crate::mixer::Card;

pub struct ProcessManager {
    jack_process: Option<Child>,
    card_processes: Vec<(i32, Child, Child)>, // ID, In, Out

    model: Model,
}


impl ProcessManager {
    pub fn new(model: Model) -> Rc<RefCell<Self>> {
        println!("process mananager new");
        let jack_process = if process_is_running("jackd") || process_is_running("jackdbus") {
            None
        } else {
            println!("starting jackd");
            let jack_proc = Command::new("jackd")
                .arg("-d")
                .arg("dummy")
                // .stdout(Stdio::piped())
                // .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start jack server");

            //wait for jack to start,
            thread::sleep(Duration::from_secs(1));
            Some(jack_proc)
        };

        let this = Rc::new(RefCell::new(Self {
            jack_process,
            card_processes: Vec::new(),
            model,
        }));

        let this_clone = this.clone();

        glib::timeout_add_local(200, move || {
            this_clone.borrow_mut().update_processes();
            Continue(true)
        });

        this
    }

    fn update_processes(&mut self) {
        let mixer = self.model.borrow().mixer.clone();
        for model_card in mixer.iter() {
            let proc_present = self.card_processes.iter().position(|x| x.0 == model_card.id).is_some();

            if !proc_present {
                self.connect_card(model_card);
            }
        }

        let mut junk_list: Vec<i32> = Vec::new();
        for card in self.card_processes.iter_mut() {
            match card.1.try_wait() {
                Ok(None) => (),
                Ok(Some(code)) => {
                    println!("Card {}: input process died {}, removing card", card.0, code);
                    card.2.kill();
                    junk_list.push(card.0); 
                },
                Err(e) => {println!("error talking to card process: {}", e)},
            }

            match card.2.try_wait() {
                Ok(None) => (),
                Ok(Some(code)) => {
                    println!("Card {}: output process died {}, removing card", card.0, code);
                    card.1.kill();
                    junk_list.push(card.0);
                },
                Err(e) => {println!("error talking to card process: {}", e)},
            }
        }

        for j in junk_list.iter() {
            match self.card_processes.iter().position(|x| x.0 == *j) {
                Some(i) => {
                    self.card_processes.remove(i);
                },
                None => (),
            }
        }
    }

    fn connect_card(&mut self, card: &Card) -> bool {
        let in_proc = Command::new("alsa_in")
        .arg("-j")
        .arg(format!("{} - Inputs",card.name()))
        .arg("-d")
        .arg(format!("hw:{}", card.id))
        .arg("-r")
        .arg("44100")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

        let out_proc = Command::new("alsa_out")
        .arg("-j")
        .arg(format!("{} - Outputs",card.name()))
        .arg("-d")
        .arg(format!("hw:{}", card.id))
        .arg("-r")
        .arg("44100")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

        if in_proc.is_ok() && out_proc.is_ok() {
            self.card_processes.push((card.id, in_proc.unwrap(), out_proc.unwrap()));
            true
        } else {
            println!("error connecting to card {}", card.id);
            false
        }
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        let _ = match &mut self.jack_process {
            Some(p) => p.kill(),
            None => Ok(()),
        };
    }
}

fn process_is_running(name: &str) -> bool {
    for process in process::processes()
        .expect("failed to list processes")
        .iter()
    {
        match process {
            Ok(process) => {
                if process.name().map_err(|_| "").unwrap() == name {
                    return true;
                }
            }
            Err(_) => {}
        }
    }
    false
}
