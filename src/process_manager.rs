use crate::model::Model;
use gtk::prelude::*;
use psutil::process;
use std::cell::RefCell;
use std::panic;
use std::process::abort;
use std::process::{Child, Command};
use std::rc::Rc;
use std::thread;
use std::time::Duration;

pub struct ProcessManager {
    jack_process: Option<Child>,
}

static mut MUT_JACKCTL_SPAWNED_SERVER: bool = false;

fn panic_kill(info: &panic::PanicInfo) -> ! {
    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    eprintln!("{}", info);

    eprintln!("Killing children (violently)");
    // We're throwing the results away cos this is a panic handler... what are we gonna do if it fails?!
    let _ = Command::new("killall").arg("-9").arg("alsa_in").spawn();
    let _ = Command::new("killall").arg("-9").arg("alsa_out").spawn();
    unsafe {
        if MUT_JACKCTL_SPAWNED_SERVER {
            eprintln!("Killing Local Server");
            let _ = Command::new("killall").arg("-9").arg("jackd").spawn();
        }
    }

    abort();
}

impl ProcessManager {
    pub fn new(_model: Model) -> Rc<RefCell<Self>> {
        panic::set_hook(Box::new(|pi| {
            panic_kill(pi);
        }));

        println!("process mananager new");
        let jack_process = if process_is_running("jackd") || process_is_running("jackdbus") {
            None
        } else {
            println!("starting jackd");
            let jack_proc = Command::new("jackd")
                // This magic incantation launches jack with no input or output ports at all
                .args(["-r", "-d", "dummy", "-C", "0", "-P", "0"].iter())
                // .stdout(Stdio::piped())
                // .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start jack server");

            //wait for jack to start,
            unsafe {
                MUT_JACKCTL_SPAWNED_SERVER = true;
            }
            thread::sleep(Duration::from_secs(1));
            Some(jack_proc)
        };

        let this = Rc::new(RefCell::new(Self { jack_process }));

        let this_clone = this.clone();

        glib::timeout_add_local(200, move || {
            this_clone.borrow_mut().update_processes();
            Continue(true)
        });

        this
    }

    fn update_processes(&mut self) {}

    pub fn end(&mut self) {
        let _ = match &mut self.jack_process {
            Some(p) => {
                println!("stopping server");
                p.kill()
            }
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
