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

/// UI event types executed on the model
#[derive(Clone, Debug)]
pub enum UiEvent {
    /// Called to reset the overrun count. (For example when the user presses a button)
    ResetXruns,
    /// Called when the user requests a mute operation on a channel
    SetMuting(CardId, ChannelId, bool),
    /// Called when the user requests a volume change on a channel
    SetVolume(CardId, ChannelId, Volume),
    /// Called when ALSA has new channel data,
    UpdateChannel(CardId, ChannelId, Volume, bool),
    /// Called to clear the dirty bit on a channel when a UI change has finished syncing
    CleanChannel(CardId, ChannelId),
}

#[derive(Debug)]
pub enum UiCmd {
    AddCard,
    RemoveCard,
    AddClient,
    RemoveClient,
}

/// Jack event types executed on the model
#[derive(Clone)]
pub enum JackEvent {
    /// Called when the JACK Server overruns
    XRun,
    /// Called when jack has new server settings.
    JackSettings(f32, u64, u64, u64),

    /// Called when the Model detects a new card to add to the model
    AddCard(CardId, String),
    /// Called to Start Enumating the Card
    UseCard(CardId),
    DontUseCard(CardId),
    FinishEnumerateCard(
        CardId,
        Option<CardConfig>,
        Option<CardConfig>,
        Vec<MixerChannel>,
    ),
    FailEnumerateCard(CardId),
    FinishStartCard(CardId, u64),
    FailStartCard(CardId),
    StopCard(CardId),
    ForgetCard(CardId),

    AddAudioInput(Port),
    AddAudioOutput(Port),
    AddMidiInput(Port),
    AddMidiOutput(Port),

    // Called when its time to delete a port,
    // 'Argument is port ID
    DelPort(JackPortType),

    AddConnection(JackPortType, JackPortType),
    DelConnection(JackPortType, JackPortType),
}
