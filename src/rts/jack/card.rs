use crate::rts::jack::JackRuntime;
use std::sync::Arc;

pub(super) async fn spawn_handle(jack: &Arc<JackRuntime>) {
    let jack = Arc::clone(jack);

    // Loop until the card_tx senders drop
    while let Ok(card) = jack.card_rx.recv().await {
        match card {
            // ...
            _ => {}
        }
    }
}
