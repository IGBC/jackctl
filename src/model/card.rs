//! Types used to define a sound card

use std::collections::HashMap;

/// Struct representing a sound card in the model
#[derive(Debug)]
pub struct Card {
    pub id: i32,
    pub inputs: Option<u32>,  // option contains best sample rate
    pub outputs: Option<u32>, // option contains best sample rate
    name: String,
    pub channels: HashMap<u32, MixerChannel>,
    pub state: CardStatus,
}

/// Defines all the state a card can be in
#[derive(Clone, Debug, PartialEq)]
pub enum CardStatus {
    /// We just found this card, we don't know anything about it yet
    New,
    /// This Card should be enumerated now
    Enum,
    /// This Card should be started now
    Start,
    /// this card is in use
    Active,
    /// This card was just stopped, it should be put back into new in a few seconds
    Stopped,
    /// This card could not be enumerated, we are going to leave it alone
    EnumFailed,
    /// This card could not be started, we are going to leave it alone
    StartFailed,
    /// This card is busy, put back to new after a timeout,
    Busy,
    /// The user has told us not to use this card
    DontUse,
}

/// Struct representing a mixer channel in the model.
/// A mixer channel is a typically a volume slider and a mute switch exposed
/// as by ALSA.
#[derive(Debug, PartialEq)]
pub struct MixerChannel {
    pub id: u32,
    name: String,

    pub is_playback: bool,
    pub has_switch: bool,
    pub volume_min: i64,
    pub volume_max: i64,

    pub volume: i64,
    pub switch: bool,

    pub dirty: bool,
}

impl MixerChannel {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

// impl fmt::Debug for MixerChannel {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("MixerChannel")
//             .field("id", &(self.id.get_index(), self.id.get_name()))
//             .field("has_switch", &self.has_switch)
//             .field("volume_max", &self.volume_max)
//             .field("volue_min", &self.volume_min)
//             .field("is_playback", &self.is_playback)
//             .finish()
//     }
// }

// impl std::cmp::PartialEq for MixerChannel {
//     fn eq(&self, other: &Self) -> bool {
//         self.id.get_index() == other.id.get_index() && self.id.get_name() == other.id.get_name()
//     }
// }

impl std::cmp::PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name && self.channels == other.channels
    }
}

impl Card {
    pub fn new(id: i32, name: String) -> Self {
        Card {
            id,
            inputs: None,
            outputs: None,
            name,
            channels: HashMap::new(),
            state: CardStatus::New,
        }
    }

    pub fn add_channel(
        &mut self,
        id: u32,
        name: String,
        is_playback: bool,
        has_switch: bool,
        volume_min: i64,
        volume_max: i64,
    ) {
        let channel = MixerChannel {
            id,
            name,
            is_playback,
            has_switch,
            volume_min,
            volume_max,
            volume: 0,
            switch: false,
            dirty: false,
        };
        self.channels.insert(id, channel);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn len(&self) -> usize {
        self.channels.len()
    }

    pub fn iter(
        &self,
    ) -> std::collections::hash_map::Values<'_, u32, crate::model::card::MixerChannel> {
        self.channels.values()
    }
}
