use gio::ResourceLookupFlags;
use gtk::prelude::*;
use gtk::Window;
use libappindicator::{AppIndicator, AppIndicatorStatus};

use std::env::temp_dir;
use std::fs::{remove_file, File};
use std::io::prelude::*;
use std::path::PathBuf;

use crate::model::events::UiEvent;
use crate::ui::UiRuntime;

pub struct TrayState {
    indicator: AppIndicator,
    icon_file: PathBuf,
}

impl TrayState {
    pub(super) fn new(rt: UiRuntime, window: Window) -> Self {
        let icon_path: PathBuf = temp_dir();
        info!("Indicator Icon Path = {}", icon_path.to_str().unwrap());

        let icon_file = icon_path.join("jackctl-symbolic.svg");
        let mut file = File::create(icon_file.clone().into_os_string()).unwrap();
        let icon = gio::resources_lookup_data(
            "/net/jackctl/Jackctl/icons/jackctl-symbolic.svg",
            ResourceLookupFlags::NONE,
        )
        .unwrap();
        file.write_all(&icon);

        let mut indicator = AppIndicator::new("jackctl", "");
        indicator.set_status(AppIndicatorStatus::Active);
        indicator.set_icon_theme_path(icon_path.to_str().unwrap());
        indicator.set_icon("jackctl-symbolic");
        let mut m = gtk::Menu::new();
        let mi = gtk::MenuItem::with_label("Show");
        mi.connect_activate(move |_| {
            window.show();
        });
        m.append(&mi);
        let mi = gtk::MenuItem::with_label("Quit");
        mi.connect_activate(move |_| rt.sender().send(UiEvent::Shutdown));
        m.append(&mi);
        indicator.set_menu(&mut m);
        m.show_all();

        Self {
            indicator,
            icon_file,
        }
    }
}

impl Drop for TrayState {
    fn drop(&mut self) {
        let _ = remove_file(self.icon_file.clone().into_os_string());
    }
}
