use std::collections::HashMap;

// TODO: make this compatbile with different audio backends
pub type CardId = i32;
pub type ChannelId = u32;
pub type Volume = i64;
pub type SampleRate = u32;
pub type ChannelCount = u32;

/// Struct representing a sound card in the model
#[derive(Debug)]
pub struct Card {
    pub id: i32,
    pub client_handle: Option<u64>,
    pub inputs: Option<CardConfig>, // option contains best sample rate
    pub outputs: Option<CardConfig>, // option contains best sample rate
    name: String,
    pub channels: HashMap<u32, MixerChannel>,
    pub state: CardStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CardConfig {
    pub sample_rate: u32,
    pub channels: u32,
}

/// Defines all the state a card can be in
#[derive(Clone, Debug, PartialEq)]
pub enum CardStatus {
    /// We just found this card, we don't know anything about it yet
    New,
    /// this card is in use
    Active,
    /// This card is busy, put back to new after a timeout,
    Busy,
    /// The user has told us not to use this card
    DontUse,
    // Both busy and don't use it
    // BusyDontUse,
}

/// Struct representing a mixer channel in the model.
/// A mixer channel is a typically a volume slider and a mute switch exposed
/// as by ALSA.
#[derive(Clone, Debug, PartialEq)]
pub struct MixerChannel {
    pub id: u32,
    pub name: String,

    pub is_playback: bool,
    pub has_switch: bool,
    pub volume_min: i64,
    pub volume_max: i64,

    pub volume: i64,
    pub switch: bool,

    pub dirty: bool,
}
