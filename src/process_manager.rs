use psutil::process;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::panic;
use std::process::abort;

use gtk::prelude::*;

use crate::model::Model;
use crate::model::{Card, CardStatus};

struct CardEntry {
    pub id: i32,
    pub in_proc: Option<Child>,
    pub out_proc: Option<Child>,
}

pub struct ProcessManager {
    jack_process: Option<Child>,
    card_processes: HashMap<i32, CardEntry>, // ID, card

    model: Model,
}

impl CardEntry {
    fn new(id: i32) -> Self {
        Self {
            id,
            in_proc: None,
            out_proc: None,
        }
    }
}

static mut jackctl_spawned_server: bool = false;


fn panic_kill(info: &panic::PanicInfo) -> ! {
    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    eprintln!("{}", info);

    eprintln!("Killing children (violently)");
    Command::new("killall").arg("-9").arg("alsa_in").spawn();
    Command::new("killall").arg("-9").arg("alsa_out").spawn();
    unsafe {
        if jackctl_spawned_server {
            eprintln!("Killing Local Server");
            Command::new("killall").arg("-9").arg("jackd").spawn();
        }
    }

    abort();
}


impl ProcessManager {
    pub fn new(model: Model) -> Rc<RefCell<Self>> {
        
        panic::set_hook(Box::new(|pi| {
            panic_kill(pi);
        }));

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
            unsafe {
                jackctl_spawned_server = true;
            }
            thread::sleep(Duration::from_secs(1));
            Some(jack_proc)
        };

        let this = Rc::new(RefCell::new(Self {
            jack_process,
            card_processes: HashMap::new(),
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
        let model = self.model.clone();
        for card in model.borrow().cards.values() {
            match card.state {
                CardStatus::Active => {
                    if !self.card_processes.contains_key(&card.id) {
                        self.card_processes.insert(card.id, CardEntry::new(card.id));
                    }

                    let proc = self.card_processes.get_mut(&card.id).unwrap();
                    if proc.in_proc.is_none() {
                        proc.in_proc = Self::connect_input(card).unwrap_or(None);
                    }

                    if proc.out_proc.is_none() {
                        proc.out_proc = Self::connect_output(card).unwrap_or(None);
                    }
                }
                _ => (),
            }
        }

        let mut junk_list: Vec<i32> = Vec::new();
        for (id, mut card) in self.card_processes.iter_mut() {
            match &mut card.in_proc {
                Some(proc) => match &mut proc.try_wait() {
                    Ok(None) => (),
                    Ok(Some(code)) => {
                        println!("Card {}: input process died {}, removing card", id, code);
                        card.in_proc = None;
                    }
                    Err(e) => println!("error talking to card process: {}", e),
                },
                None => (),
            }

            match &mut card.out_proc {
                Some(proc) => match &mut proc.try_wait() {
                    Ok(None) => (),
                    Ok(Some(code)) => {
                        println!("Card {}: output process died {}, removing card", id, code);
                        card.out_proc = None;
                    }
                    Err(e) => println!("error talking to card process: {}", e),
                },
                None => (),
            }
        }
    }

    fn connect_input(card: &Card) -> io::Result<Option<Child>> {
        let in_proc = match card.inputs {
            Some(rate) => Some(
                Command::new("alsa_in")
                    .arg("-j")
                    .arg(format!("{} - Inputs", card.name()))
                    .arg("-d")
                    .arg(format!("hw:{}", card.id))
                    .arg("-r")
                    .arg(format!("{}", rate))
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?,
            ),
            None => None,
        };
        Ok(in_proc)
    }

    fn connect_output(card: &Card) -> io::Result<Option<Child>> {
        let out_proc = match card.outputs {
            Some(rate) => Some(
                Command::new("alsa_out")
                    .arg("-j")
                    .arg(format!("{} - Outputs", card.name()))
                    .arg("-d")
                    .arg(format!("hw:{}", card.id))
                    .arg("-r")
                    .arg(format!("{}", rate))
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?,
            ),
            None => None,
        };
        Ok(out_proc)
    }

    pub fn end(&mut self) {
        for card in self.card_processes.values_mut() {
            println!("releasing card {}", card.id);
            let _ = match &mut card.in_proc {
                Some(p) => p.kill(),
                None => Ok(()),
            };

            let _ = match &mut card.out_proc {
                Some(p) => p.kill(),
                None => Ok(()),
            };
        }

        let _ = match &mut self.jack_process {
            Some(p) => {
                println!("stopping server");
                p.kill()
            },
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
                let proc_name = match process.name() {
                    Ok(n) => n,
                    Err(_) => "".to_owned(),
                };
                if proc_name == name {
                    return true;
                }
            }
            Err(_) => {}
        }
    }
    false
}
