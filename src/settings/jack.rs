use serde::{Deserialize, Serialize};

/// Jack server settings
#[derive(Debug, Serialize, Deserialize)]
pub struct JackSettings {
    /// Keep track of the version of this file
    version: u8,
    /// Specify how the jack server is launched
    pub spawn_mode: SpawnMode,
    /// How should the jack server behave
    pub run_mode: RunMode,
    /// Enable jack realtime mode
    pub realtime: bool,
    /// Specify audio block size
    pub block_size: u32,
    /// Specify server sample rate
    pub sample_rate: u32,
}

impl Default for JackSettings {
    fn default() -> Self {
        // very controvertial default values
        Self {
            version: 1,
            spawn_mode: SpawnMode::SoftSpawn,
            run_mode: RunMode::Uninitialized,
            realtime: false,
            block_size: 1024,
            sample_rate: 48000,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SpawnMode {
    /// Wait for jack to be spawned
    Wait,
    /// Attempt to spawn jack but fall back to existing servers
    SoftSpawn,
    /// Force spawn jack and kill competing servers
    ForceSpawn,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RunMode {
    /// This setting has not been initialised by the user yet
    Uninitialized,
    /// Ignore PA, ask for every card
    Ignore,
    /// Spawn PA module to bridge with
    BridgePA,
    /// Run Jack as a PA client (ish)
    BridgeJack,
    /// Secret option (pipewire)
    Pipewire,
}
