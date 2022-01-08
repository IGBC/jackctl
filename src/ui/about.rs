use std::collections::BTreeMap;

use crate::ui::utils;
use gtk::{AboutDialog, AboutDialogExt, Builder, ButtonExt, DialogExt, ModelButton, WidgetExt};

pub struct About {
    inner: AboutDialog,
}

const CONTRIBUTORS: &'static str = include_str!("../../contributors.toml");
type ContributorMap = BTreeMap<String, BTreeMap<String, String>>;

fn unpack_map(map: Option<&BTreeMap<String, String>>) -> Vec<String> {
    map.map(|map| map.iter().map(|(k, v)| format!("{} <{}>", k, v)).collect())
        .unwrap_or_default()
}

impl About {
    /// Create the main about dialog builder
    pub fn new(b: &Builder) -> Self {
        let inner: AboutDialog = utils::get_object(b, "aboutdialog");

        // Parse contributors
        match toml::from_str::<ContributorMap>(&CONTRIBUTORS) {
            Ok(c) => {
                let devers = unpack_map(c.get("developers"));
                let artoids = unpack_map(c.get("artists"));
                let transes = unpack_map(c.get("translators"));

                inner.set_authors(
                    devers
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .as_slice(),
                );
                inner.set_artists(
                    artoids
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .as_slice(),
                );
                inner.set_translator_credits(Some(transes.join("\n").as_str()));
            }
            Err(e) => {
                error!("Failed to parse contributors: {:?}", e)
            }
        }

        inner.set_version(Some(env!("CARGO_PKG_VERSION")));
        inner.connect_response(move |dialog, _| dialog.hide());
        Self { inner }
    }

    /// Initialise a button to open the about dialog
    pub fn button(self, b: &Builder) {
        let button: ModelButton = utils::get_object(b, "about.mainmenu");
        button.connect_clicked(move |_| self.inner.show());
    }
}
