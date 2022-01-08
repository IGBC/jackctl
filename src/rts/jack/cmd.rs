use crate::model::{
    events::{JackCmd, JackEvent, JackSettings},
    port::JackPortType,
};
use crate::rts::jack::JackRuntime;
use async_std::task;
use jack::Client;
use std::{sync::Arc, time::Duration};

pub async fn do_event(jack: Arc<JackRuntime>) {
    loop {
        if jack.cmd_rx.is_closed() {
            break;
        }

        let settings = interval_update(&jack);

        jack.event_tx
            .send(JackEvent::JackSettings(settings))
            .await
            .unwrap();

        // this rate limits updates to the mixers, we don't need to update at 100 FPS
        task::sleep(Duration::from_millis(100)).await;
    }
}

pub async fn spawn_handle(jack: Arc<JackRuntime>) {
    // Loop until the card_tx senders drop
    while let Ok(cmd) = jack.cmd_rx.recv().await {
        println!("Handling jack client event...");
        match cmd {
            JackCmd::ConnectPorts {
                input,
                output,
                connect,
            } => {
                connect_ports(&jack.a_client.as_client(), output, input, connect);
                println!("Connect ports...");
            }
            JackCmd::Shutdown => {
                break;
            }
        }
    }
}

/// Connect two jack ports together on the server.
fn connect_ports(client: &Client, input: JackPortType, output: JackPortType, connect: bool) {
    let i = client.port_by_id(input);
    let o = client.port_by_id(output);
    if i.is_some() && o.is_some() {
        let o = o.unwrap();
        let i = i.unwrap();
        let result = if connect {
            client.connect_ports(&o, &i)
        } else {
            client.disconnect_ports(&o, &i)
        };
        if result.is_err() {
            println!("Connection Error: {}", result.unwrap_err());
        }
    } else {
        println!("Failed to create connection {}->{}, one of the ports is missing", input, output);
    }
}

fn interval_update(jack: &Arc<JackRuntime>) -> JackSettings {
    let client = jack.a_client.as_client();
    let cpu_percentage = client.cpu_load();
    let sample_rate = client.sample_rate() as u64;
    let buffer_size = client.buffer_size() as u64;
    let latency = (buffer_size) as f32 / (sample_rate as f32 / 1000.0) * jack.n_periods as f32;

    JackSettings {
        cpu_percentage,
        sample_rate,
        buffer_size,
        latency,
    }
}
