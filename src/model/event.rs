//! Event types used for controlling the Model.

use super::{CardConfig, JackPortType, MixerChannel, Port};
use crate::mixer::{CardId, ChannelId, Volume};

/// Event type represents methods that can be called on the model.
pub enum Event {
    /// Called when the JACK Server overruns
    XRun,
    /// Called to reset the overrun count. (For example when the user presses a button)
    ResetXruns,
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

    /// Called when the user requests a mute operation on a channel
    SetMuting(CardId, ChannelId, bool),
    /// Called when the user requests a volume change on a channel
    SetVolume(CardId, ChannelId, Volume),
    /// Called when ALSA has new channel data,
    UpdateChannel(CardId, ChannelId, Volume, bool),
    /// Called to clear the dirty bit on a channel when a UI change has finished syncing
    CleanChannel(CardId, ChannelId),

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
