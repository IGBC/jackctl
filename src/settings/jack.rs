use serde::{Deserialize, Serialize};

/// Jack server settings
#[derive(Serialize, Deserialize)]
pub struct JackSettings {
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
            spawn_mode: SpawnMode::SoftSpawnJack,
            run_mode: RunMode::Uninitialized,
            realtime: false,
            block_size: 1024,
            sample_rate: 48000,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum SpawnMode {
    /// Only bridge Jack to pulse-audio
    PaBridge,
    /// Wait for jack to be spawned
    WaitForJack,
    /// Attempt to spawn jack but fall back to existing servers
    SoftSpawnJack,
    /// Force spawn jack and kill competing servers
    ForceSpawnJack,
}

#[derive(Serialize, Deserialize)]
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
