//! Event types used for controlling the Model.

use crate::mixer::{CardId, ChannelId, Volume};
use super::{Connection, Port, JackPortType};

/// Event type represents methods that can be called on the model.
pub enum Event<'a> {
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

    AddAudioInput(Port),
    AddAudioOutput(Port),
    AddMidiInput(Port),
    AddMidiOutput(Port),

    // Called when its time to delete a port,
    // 'Argument is port ID 
    DelPort(JackPortType),
    /// Called when the Jack controller wants to overwrite all of the existing connections
    SyncConnections(Vec<Connection<'a>>),

    AddAudioConnection(Connection<'a>),
    DelAudioConnection(Connection<'a>),

    AddMidiConnection(Connection<'a>),
    DelMidiConnection(Connection<'a>),
}