use std::collections::HashMap;

#[derive(Debug)]
pub struct Card {
    pub id: i32,
    pub inputs: Option<u32>,  // option contains best sample rate
    pub outputs: Option<u32>, // option contains best sample rate
    name: String,
    pub channels: HashMap<u32, MixerChannel>,
    pub state: CardStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CardStatus {
    Unknown,
    Active,
    Busy,
    EnumFailed,
    DontUse,
}

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
            state: CardStatus::Unknown,
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
