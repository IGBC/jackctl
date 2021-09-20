//! Event types used for controlling the Model.

use crate::mixer::{CardId, ChannelId, Volume};
use super::Connection;

/// Event type represents methods that can be called on the model.
pub enum Event {
    /// Called when the JACK Server overruns
    XRun,
    /// Called to reset the overrun count. (For example when the user presses a button)
    ResetXruns,
    /// Called when the Model detects a new card to add to the model
    AddCard(CardId, String),
    /// Called when the user requests a mute operation on a channel
    SetMuting(CardId, ChannelId, bool),
    /// Called when the user requests a volume change on a channel
    SetVolume(CardId, ChannelId, Volume),

    AddAudioInput(String),
    DelAudioInput(String),
    /// Called when the Jack controller wants to overwrite all of the existing connections
    SyncAudioInputs(Vec<String>),
    AddAudioOutput(String),
    DelAudioOutput(String),
    /// Called when the Jack controller wants to overwrite all of the existing connections
    SyncAudioOutputs(Vec<String>),

    AddMidiInput(String),
    DelMidiInput(String),
    /// Called when the Jack controller wants to overwrite all of the existing connections
    SyncMidiInputs(Vec<String>),
    AddMidiOutput(String),
    DelMidiOutput(String),
    /// Called when the Jack controller wants to overwrite all of the existing connections
    SyncMidiOutputs(Vec<String>),

    /// Called when the Jack controller wants to overwrite all of the existing connections
    SyncConnections(Vec<Connection>),

    AddAudioConnection(Connection),
    DelAudioConnection(Connection),

    AddMidiConnection(Connection),
    DelMidiConnection(Connection),
}