use crate::model2::card::{CardId, ChannelId, Volume};

//ifconfig is linux
mod alsa_card;

pub use alsa_card::AlsaHandle as HardwareHandle;

// ifconfig is mac
// mod coraudio;

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

pub enum HardwareEvent {
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

pub enum HardwareCardAction {}
