use serde::{Deserialize, Serialize};

/// jackctl application settings tree
///
/// These settings modify the base behaviour of the application and
/// user preferences.
#[derive(Serialize, Deserialize)]
pub struct AppSettings {
    /// How the UI opens initially
    pub ui_launch_mode: UiLaunchMode,
    /// The base mode for jackctl
    pub base_mode: BaseMode,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ui_launch_mode: UiLaunchMode::Wizard,
            base_mode: BaseMode::SoftSpawnJack,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum UiLaunchMode {
    /// Open the setup wizard
    Wizard,
    /// Open the main window
    Open,
    /// Immediately minimise to tray
    Tray,
}

#[derive(Serialize, Deserialize)]
pub enum BaseMode {
    /// Only bridge Jack to pulse-audio
    PaBridge,
    /// Wait for jack to be spawned
    WaitForJack,
    /// Attempt to spawn jack but fall back to existing servers
    SoftSpawnJack,
    /// Force spawn jack and kill competing servers
    ForceSpawnJack,
}
