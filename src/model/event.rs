use crate::mixer::{CardId, ChannelId, Volume};
use super::Connection;


pub enum Event {
    XRun,
    ResetXruns,
    AddCard(CardId, String),
    SetMuting(CardId, ChannelId, bool),
    SetVolume(CardId, ChannelId, Volume),

    AddAudioInput(String),
    DelAudioInput(String),
    SyncAudioInputs(Vec<String>),
    AddAudioOutput(String),
    DelAudioOutput(String),
    SyncAudioOutputs(Vec<String>),

    AddMidiInput(String),
    DelMidiInput(String),
    SyncMidiInputs(Vec<String>),
    AddMidiOutput(String),
    DelMidiOutput(String),
    SyncMidiOutputs(Vec<String>),

    SyncConnections(Vec<Connection>),

    AddAudioConnection(Connection),
    DelAudioConnection(Connection),

    AddMidiConnection(Connection),
    DelMidiConnection(Connection),
}