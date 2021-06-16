use psutil::process;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;

use crate::model::Model;
use crate::mixer::Card;

pub struct ProcessManager {
    jack_process: Option<Child>,
    card_processes: Vec<(i32,Child)>,

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
    }

    fn connect_card(&mut self, card: &Card) -> bool {
        let proc = Command::new("alsa_out")
        .arg("-j")
        .arg(card.name())
        .arg("-d")
        .arg(format!("hw:{}", card.id))
        .arg("-r")
        .arg("44100")
        // .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .spawn();

        match proc {
            Ok(c) => {
                self.card_processes.push((card.id, c));
                true
            },
            Err(e) => {
                println!("error connecting to card {}: {}", card.id, e);
                false
            }
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
