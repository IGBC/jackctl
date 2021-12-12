use crate::{model::card::CardUsage, settings::Id};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Remember audio devices previously configured with jackctl
///
///
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CardSettings {
    /// Store all known card settings
    known: BTreeMap<String, SoundCard>,
    /// Identify a "default" sound card
    default: Id,
}

impl CardSettings {
    pub fn set_card_usage(&mut self, name: &String, _use: bool) {
        match self.known.get_mut(name) {
            Some(entry) => entry._use = _use,
            None => {
                self.known.insert(
                    name.clone(),
                    SoundCard {
                        name: name.clone(),
                        _use,
                    },
                );
            }
        }

        println!("{:?}", self.known);
    }

    pub fn use_card(&self, name: &String) -> CardUsage {
        match self.known.get(name) {
            Some(card) if card._use => CardUsage::Yes,
            Some(_) => CardUsage::No,
            None => CardUsage::AskUser,
        }
    }
}

/// Encoding information about a single sound card
#[derive(Debug, Serialize, Deserialize)]
struct SoundCard {
    pub name: String,
    pub _use: bool,
}
