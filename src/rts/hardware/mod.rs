use crate::model2::card::{CardConfig, CardId, ChannelId, MixerChannel, Volume};

//ifconfig is linux
mod alsa_card;

pub use alsa_card::AlsaHandle as HardwareHandle;

// ifconfig is mac
// mod coraudio;

#[derive(Debug)]
pub enum HardwareCmd {
    SetMixerVolume {
        card: CardId,
        channel: ChannelId,
        volume: Volume,
    },

    SetMixerMute {
        card: CardId,
        channel: ChannelId,
        mute: bool,
    },
}

#[derive(Debug)]
pub enum HardwareEvent {
    NewCardFound {
        id: CardId,
        capture: Option<CardConfig>,
        playback: Option<CardConfig>,
        mixerchannels: Vec<MixerChannel>,
    },

    DropCard {
        id: CardId,
    },

    UpdateMixerVolume {
        card: CardId,
        channel: ChannelId,
        volume: Volume,
    },

    UpdateMixerMute {
        card: CardId,
        channel: ChannelId,
        mute: bool,
    },
}

#[derive(Clone, Debug)]
pub enum HardwareCardAction {}
