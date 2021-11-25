use once_cell::sync::OnceCell;
use psutil::process;
use std::panic;
use std::process::abort;
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct JackServer {
    jack_process: Option<Child>,
}

static JACKCTL_SPAWNED_SERVER: OnceCell<bool> = OnceCell::new();

fn panic_kill(info: &panic::PanicInfo) -> ! {
    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    eprintln!("{}", info);

    if *JACKCTL_SPAWNED_SERVER.get().unwrap_or(&false) {
        eprintln!("Killing Local Server");
        let _ = Command::new("killall").arg("-9").arg("jackd").spawn();
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
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start jack server");

            // wait for a moment for the server to start else the client might start first
            thread::sleep(Duration::from_millis(100));
            Some(jack_proc)
        };

        let _ = JACKCTL_SPAWNED_SERVER.set(jack_process.is_some()); // we don't actually care

        Self { jack_process }
    }

    pub fn end(&mut self) {
        match &mut self.jack_process {
            Some(p) => {
                println!("stopping server");
                let _ = p.kill();
                let _ = p.wait();
            }
            None => (),
        }
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
        println!("Dropping jack server");
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

#[allow(unused)]
#[cfg(test)]
mod tests {
    use jack::{Client, PortFlags};

    // this ensures only one of these tests runs at once;
    use std::sync::{Mutex, MutexGuard};
    static SERVER_MUTEX: super::OnceCell<Mutex<()>> = super::OnceCell::new();

    fn setup_test<'a>() -> MutexGuard<'a, ()> {
        let _ = SERVER_MUTEX.set(Mutex::new(())); // or don't we expect this to fail a lot;
        match SERVER_MUTEX.get() {
            Some(mutex) => match mutex.lock() {
                Ok(g) => {
                    return g;
                }
                Err(e) => {
                    assert!(false, "Test mutex is poisoned {}", e);
                }
            },
            None => {
                assert!(false, "Test mutex is poisoned");
            }
        }
        panic!();
    }

    fn launch_server() -> super::JackServer {
        if super::process_is_running("jackd") || super::process_is_running("jackdbus") {
            panic!("Ensure jack server is off before running tests");
        }

        super::JackServer::new(44100, 512, false)
    }

    #[test]
    fn check_server_launches() {
        let mutex = setup_test();
        let mut server = launch_server();
        assert!(super::process_is_running("jackd"));
        server.end();
    }

    #[test]
    fn check_server_drops() {
        let mutex = setup_test();
        {
            let v = launch_server();
            assert!(super::process_is_running("jackd"));
        }
        assert!(!super::process_is_running("jackd"));
    }

    #[test]
    fn check_get_stdout() {
        let mutex = setup_test();
        let mut server = launch_server();
        assert!(server.stdout().is_some());
        server.end();
    }

    #[test]
    fn check_get_stderr() {
        let mutex = setup_test();
        let mut server = launch_server();
        assert!(server.stderr().is_some());
        server.end();
    }

    #[test]
    fn check_settings() {
        let mutex = setup_test();
        let v = launch_server();
        let (client, _) =
            Client::new("jackctl_tests", jack::ClientOptions::NO_START_SERVER).unwrap();
        assert_eq!(client.sample_rate(), 44100);
        assert_eq!(client.buffer_size(), 512);
    }

    #[test]
    fn check_no_dummy_ports() {
        let mutex = setup_test();
        let v = launch_server();
        let (client, _) =
            Client::new("jackctl_tests", jack::ClientOptions::NO_START_SERVER).unwrap();
        //get all ports
        let ports = client.ports(None, None, PortFlags::empty());
        //check there are none
        assert!(ports.is_empty());
    }
}
