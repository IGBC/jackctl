use crate::model2::events::JackCmd;
use crate::rts::jack::JackRuntime;
use std::sync::Arc;

pub async fn spawn_handle(jack: Arc<JackRuntime>) {
    // Loop until the jack cmd sender drops
    while let Ok(cmd) = jack.cmd_rx.recv().await {
        println!("Handling jack base command...");
        match cmd {
            JackCmd::ConnectPorts {
                input,
                output,
                connect,
            } => {}
            Shutdown => {
                break;
            }
        }
    }
}
