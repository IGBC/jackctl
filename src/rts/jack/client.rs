use crate::model2::events::JackCmd;
use crate::rts::jack::JackRuntime;
use jack::Client;
use std::sync::Arc;

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
                connect_ports(&jack.a_client.as_client(), &input, &output, connect);
                println!("Connect ports...");
            }
            Shutdown => {
                break;
            }
        }
    }
}

/// Connect two jack ports together on the server.
fn connect_ports(client: &Client, input: &str, output: &str, connect: bool) {
    let result = if connect {
        client.connect_ports_by_name(&output, &input)
    } else {
        client.disconnect_ports_by_name(&output, &input)
    };
    if result.is_err() {
        println!("Connection Error: {}", result.unwrap_err());
    }
}

fn interval_update(client: &Client) {
    let cpu_percent = client.cpu_load();
    let sample_rate = client.sample_rate();
    let buffer_size = client.buffer_size();
    let latency = (buffer_size) as u64 / (sample_rate as u64 / 1000u64) as u64;

    todo!("write this back to the model")
}
