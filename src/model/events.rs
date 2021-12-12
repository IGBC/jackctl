use crate::{
    model::card::{Card, CardConfig, CardId, ChannelId, MixerChannel, Volume},
    model::port::{JackPortType, Port},
};
use jack::InternalClientID;

/// A general jack action
#[derive(Clone)]
pub enum JackCmd {
    ConnectPorts {
        input: JackPortType,
        output: JackPortType,
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
    pub cpu_percentage: f32,
    pub sample_rate: u64,
    pub buffer_size: u64,
    pub latency: f32,
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
    /// The user told us about their sound card
    CardUsage {
        card: Card,
        usage: bool,
        store: bool,
    },
    /// Add a connection between two ports
    SetConnection(JackPortType, JackPortType, bool),
    /// The user has requested the program to end
    Shutdown,
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
    /// Tell Mixer we found a new sound card
    AddCard(Card),
    /// Tell Mixer we lost a card
    DelCard(CardId),
    /// Ask the user about their sound card
    AskCard(Card),
    /// The Model Has finished a shutdown request the main loop must be terminated immediately
    YouDontHaveToGoHomeButYouCantStayHere,
}

#[derive(Clone, Debug)]
pub enum HardwareCmd {
    SetMixerVolume(VolumeCmd),
    SetMixerMute(MuteCmd),
    Shutdown,
}

#[derive(Clone, Debug)]
pub enum HardwareEvent {
    NewCardFound {
        id: CardId,
        name: String,
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
