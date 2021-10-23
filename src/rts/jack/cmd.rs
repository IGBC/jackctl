use crate::rts::jack::JackRuntime;
use std::sync::Arc;

pub(super) async fn spawn_handle(jack: &Arc<JackRuntime>) {
    let jack = Arc::clone(jack);

    // Loop until the jack cmd sender drops
    while let Ok(cmd) = jack.cmd_rx.recv().await {
        match cmd {
            // ...
            _ => {}
        }
    }
}
