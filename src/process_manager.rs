use psutil::process;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

pub struct ProcessManager {
    jack_process: Option<Child>,
}

impl ProcessManager {
    pub fn new() -> Self {
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

        ProcessManager { jack_process }
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
