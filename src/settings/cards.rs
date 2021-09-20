use crate::settings::Id;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Remember audio devices previously configured with jackctl
///
///
#[derive(Default, Serialize, Deserialize)]
pub struct CardSettings {
    /// Store all known card settings
    pub known: BTreeMap<Id, SoundCard>,
    /// Identify a "default" sound card
    pub default: Id,
}

/// Encoding information about a single sound card
#[derive(Serialize, Deserialize)]
pub struct SoundCard {
    pub name: String,
}
