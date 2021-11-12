use once_cell::sync::OnceCell;
use psutil::process;
use std::panic;
use std::process::abort;
use std::process::{Child, ChildStderr, ChildStdout, Command};
use std::thread;
use std::time::Duration;

pub struct JackServer {
    jack_process: Option<Child>,
}

static JACKCTL_SPAWNED_SERVER: OnceCell<bool> = OnceCell::new();

fn panic_kill(info: &panic::PanicInfo) -> ! {
    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    eprintln!("{}", info);

    unsafe {
        if *JACKCTL_SPAWNED_SERVER.get().unwrap_or(&false) {
            eprintln!("Killing Local Server");
            let _ = Command::new("killall").arg("-9").arg("jackd").spawn();
        }
    }

    abort();
}

impl JackServer {
    pub fn new(rate: u32, frames: u32, realtime: bool) -> Self {
        panic::set_hook(Box::new(|pi| {
            panic_kill(pi);
        }));

        println!("process mananager new");
        let jack_process = if process_is_running("jackd") || process_is_running("jackdbus") {
            None
        } else {
            // get the flag needed for realtime mode and a modifier for logging
            let (r_flag, r_msg) = if realtime { ("-R", "") } else { ("-r", "out") };

            println!(
                "starting jackd at {}Hz @{} frames with{} realtime",
                rate, frames, r_msg
            );
            let jack_proc = Command::new("jackd")
                // This magic incantation launches jack with no input or output ports at all
                .args(
                    [
                        r_flag,
                        "-d",
                        "dummy",
                        "-C",
                        "0",
                        "-P",
                        "0",
                        "-r",
                        &rate.to_string(),
                        "-p",
                        &frames.to_string(),
                    ]
                    .iter(),
                )
                //.stdout(Stdio::piped())
                //.stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start jack server");

            thread::sleep(Duration::from_secs(1));
            Some(jack_proc)
        };

        JACKCTL_SPAWNED_SERVER.set(jack_process.is_some()).unwrap();

        Self { jack_process }
    }

    pub fn end(&mut self) {
        let _ = match &mut self.jack_process {
            Some(p) => {
                println!("stopping server");
                p.kill()
            }
            None => Ok(()),
        };
        self.jack_process = None;
    }

    pub fn stderr(&mut self) -> Option<ChildStderr> {
        match &mut self.jack_process {
            Some(p) => p.stderr.take(),
            None => None,
        }
    }

    pub fn stdout(&mut self) -> Option<ChildStdout> {
        match &mut self.jack_process {
            Some(p) => p.stdout.take(),
            None => None,
        }
    }
}

impl Drop for JackServer {
    fn drop(&mut self) {
        self.end();
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
