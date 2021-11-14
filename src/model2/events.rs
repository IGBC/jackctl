use crate::{
    model2::card::{CardConfig, CardId, ChannelId, MixerChannel, Volume},
    model2::port::{JackPortType, Port},
};
use jack::InternalClientID;

/// A general jack action
#[derive(Clone)]
pub enum JackCmd {
    ConnectPorts {
        input: String,
        output: String,
        connect: bool,
    },
    Shutdown,
}

/// Actions taken on a soundcard
#[derive(Clone, Debug)]
pub enum JackCardAction {
    StartCard {
        id: String,
        name: String,
        rate: u32,
        in_ports: u32,
        out_ports: u32,
    },
    StopCard {
        id: InternalClientID,
    },
}

#[derive(Clone, Debug)]
pub struct JackSettings {
    cpu_percentage: f32,
    sample_rate: u64,
    buffer_size: u64,
    latency: f32,
}

#[derive(Clone, Debug)]
pub struct MuteCmd {
    pub card: CardId,
    pub channel: ChannelId,
    pub mute: bool,
}

#[derive(Clone, Debug)]
pub struct VolumeCmd {
    pub card: CardId,
    pub channel: ChannelId,
    pub volume: Volume,
}

/// UI event types executed on the model
#[derive(Clone, Debug)]
pub enum UiEvent {
    /// Called when the user requests a mute operation on a channel
    SetMuting(MuteCmd),
    /// Called when the user requests a volume change on a channel
    SetVolume(VolumeCmd),
}

/// Commands from the model to manipulate the UI state
#[derive(Clone, Debug)]
pub enum UiCmd {
    /// Add a single port to the audio/ midi matrix
    AddPort(Port),
    /// Delete a port
    DelPort(JackPortType),
    /// Changing volume on a channel
    VolumeChange(VolumeCmd),
    /// Toggle mute on a channel
    MuteChange(MuteCmd),
    /// Increment the XRun count
    IncrementXRun,
    /// Update jack settings
    JackSettings(JackSettings),
    /// Add a connection between ports
    AddConnection(JackPortType, JackPortType),
    /// Delete a connection between ports
    DelConnection(JackPortType, JackPortType),
}

/// Jack event types executed on the model
#[derive(Clone, Debug)]
pub enum JackEvent {
    /// Called when the JACK Server overruns
    XRun,
    /// Called when jack has new server settings.
    JackSettings(JackSettings),
    /// Add a port duh
    AddPort(Port),
    /// Delete a port
    DelPort(JackPortType),
    /// Add a connection between ports
    AddConnection(JackPortType, JackPortType),
    /// Delete a connection between ports
    DelConnection(JackPortType, JackPortType),
}

#[derive(Clone, Debug)]
pub enum HardwareCmd {
    SetMixerVolume(VolumeCmd),
    SetMixerMute(MuteCmd),
}

#[derive(Clone, Debug)]
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

    UpdateMixerVolume(VolumeCmd),

    UpdateMixerMute(MuteCmd),
}

#[derive(Clone, Debug)]
pub enum HardwareCardAction {}
