use crate::settings::Id;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Store Jack client settings
///
/// Don't ask the user to configure their DAW twice...
#[derive(Default, Serialize, Deserialize)]
pub struct ClientSettings {
    /// A set of clients previously configured
    pub clients: BTreeMap<Id, Client>,
}

#[derive(Serialize, Deserialize)]
pub struct Client {
    pub name: String,
}
