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
    /// Specify the order of inputs and outputs
    #[serde(rename = "input_direction")]
    pub io_order: IoOrder,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ui_launch_mode: UiLaunchMode::Wizard,
            jack: JackSettings::default(),
            io_order: IoOrder::VerticalInputs,
        }
    }
}

/// Specify the launch mode of the UI
#[derive(Debug, Serialize, Deserialize)]
pub enum UiLaunchMode {
    /// Open the setup wizard
    Wizard,
    /// Open the main window
    Open,
    /// Immediately minimise to tray
    Tray,
}

/// Specify in which order the inputs and outputs are displayed
#[derive(Debug, Serialize, Deserialize)]
pub enum IoOrder {
    /// Inputs are on top, horizontally spread across the UI
    #[serde(rename = "horizontal")]
    HorizontalInputs,
    /// Inputs are on the right, vertically spread across the UI
    #[serde(rename = "vertical")]
    VerticalInputs,
}
