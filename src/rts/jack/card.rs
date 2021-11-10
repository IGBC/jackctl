use crate::rts::jack::JackRuntime;
use crate::model2::events::JackCardAction;
use jack::{Client, InternalClientID};
use std::sync::Arc;

pub async fn spawn_handle(jack: Arc<JackRuntime>) {
    println!("Card handle...");
    
    // Loop until the card_tx senders drop
    // while let Ok(card) = jack.card_rx.recv().await {
    //     match card {
    //         (JackCardAction::StartCard{
    //             id,
    //             name,
    //             in_ports,
    //             out_ports,
    //             rate,
    //             nperiods,
    //             quality,
    //         }, r) => {
    //             let result = launch_card(&jack.client, &id, &name, rate, in_ports, out_ports, nperiods, quality);
    //             r.reply(result);
    //         }
    //         (JackCardAction::StopCard{id}, r) => {
    //             stop_card(&jack.client, id);
    //             r.reply(Ok(0));
    //         }
    //     }
    // }
}

fn launch_card(
    client: &Client,
    id: &str,
    name: &str,
    rate: u32,
    in_ports: u32,
    out_ports: u32,
    nperiods: u32,
    quality: u32,
) -> Result<InternalClientID, jack::Error> {
    let psize = client.buffer_size();
    let args = format!(
        "-d {} -r {} -p {} -n {} -q {} -i {} -o {}",
        id, rate, psize, nperiods, quality, in_ports, out_ports
    );
    eprintln!("running audioadapter with: {}", args);
    eprintln!("jack_load \"{}\" audioadapter -i \"{}\"", name, args);
    client.load_internal_client(name, "audioadapter", &args)
}

fn stop_card(client: &Client, id: InternalClientID) {
    let result = client.unload_internal_client(id);
    if result.is_err() {
        panic!("Failed to Stop card: {}", result.unwrap_err());
    }
}
