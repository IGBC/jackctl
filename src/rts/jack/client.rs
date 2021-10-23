use crate::rts::jack::JackRuntime;
use std::sync::Arc;

pub(super) async fn spawn_handle(jack: &Arc<JackRuntime>) {
    let jack = Arc::clone(jack);

    // Do something
    todo!()
}
