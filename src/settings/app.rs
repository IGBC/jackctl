use crate::settings::jack::JackSettings;
use serde::{Deserialize, Serialize};

/// jackctl application settings tree
///
/// These settings modify the base behaviour of the application and
/// user preferences.
#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    /// How the UI opens initially
    pub ui_launch_mode: UiLaunchMode,
    /// Jack server settings
    pub jack: JackSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ui_launch_mode: UiLaunchMode::Wizard,
            jack: JackSettings::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UiLaunchMode {
    /// Open the setup wizard
    Wizard,
    /// Open the main window
    Open,
    /// Immediately minimise to tray
    Tray,
}
