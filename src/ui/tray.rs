use libappindicator::{AppIndicator, AppIndicatorStatus};
use gtk::prelude::*;
use gtk::Window;
use std::path::Path;

use crate::ui::UiRuntime;
use crate::model::events::UiEvent;

pub struct TrayState {
    indicator: AppIndicator,
}

impl TrayState {
    pub(super) fn new(rt: UiRuntime, window: Window) -> Self {
        let icon_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

        let mut indicator = AppIndicator::new("jackctl", "");
        indicator.set_status(AppIndicatorStatus::Active);
        indicator.set_icon_theme_path(icon_path.to_str().unwrap());
        indicator.set_icon_full("jackctl-symbolic", "icon");
        let mut m = gtk::Menu::new();
        let mi = gtk::CheckMenuItem::with_label("exit");
        mi.connect_activate(move |_| rt.sender().send(UiEvent::Shutdown));
        m.append(&mi);
        let mi = gtk::CheckMenuItem::with_label("show");
        mi.connect_activate(move |_| {
            window.show();
        });
        m.append(&mi);
        indicator.set_menu(&mut m);
        m.show_all();

        Self {
            indicator,
        }
    }
}