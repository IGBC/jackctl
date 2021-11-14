use crate::model2::events::JackCmd;
use crate::rts::jack::JackRuntime;
use jack::Client;
use std::sync::Arc;

pub async fn spawn_handle(jack: Arc<JackRuntime>) {
    //todo fire events maybe?
}

fn interval_update(client: &Client) {
    let cpu_percent = client.cpu_load();
    let sample_rate = client.sample_rate();
    let buffer_size = client.buffer_size();
    let latency = (buffer_size) as u64 / (sample_rate as u64 / 1000u64) as u64;

    todo!("write this back to the model")
}
