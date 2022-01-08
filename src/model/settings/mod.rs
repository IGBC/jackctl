//! JackCTL settings handling
//!
//! ## Application/ user settings
//!
//! - GUI behaviour changes
//! - Modify jack behaviour
//! - Define run mode (pa-bridge mode, jack-service spawning, force-spawn, etc)
//!
//! ## Jack client storage
//!
//! - Remember bitwig settings (for example)
//! - Patchbay persistance
//! - Toggle on/off via app/user settings
//!
//! ## Sound card storage
//!
//! - Remember which audio devices have been configured before
//! - Don't ask the user to configure the same device twice
//!
//! ## Usage
//!
//! First initialise the settings tree by calling
//! `Settings::init(...)`, giving it a path under which the
//! configurations are stored.  Afterwards you can access settings via
//! [`Settings::r()`](Settings::r()) and
//! [`Settings::w()`](Settings::w()).
//!
//! ```rust
//! # use crate::settings::*;
//! # fn() -> Result<(), crate::error::SettingsError> {
//! let s = Settings::init(".config/jackctl/")?;
//! println!("{:?}", s.r().app().ui_launch_mode);
//! # }
//! ```
//!
//! After applying changes to the settings, don't forget to call
//! [`sync()`](Settings::sync)!

mod app;
pub use app::{IoOrder, UiLaunchMode};

mod cards;
mod clients;
mod jack;

use crate::error::SettingsError;
use directories::ProjectDirs;
use serde::de::DeserializeOwned;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

/// Use a simple u64 to identify audio devices in the future
pub type Id = u64;

/// Create the required directories
pub fn scaffold() -> ProjectDirs {
    let dir = ProjectDirs::from("tech", "sigsegv", "jackctl").unwrap();
    let _ = fs::create_dir(&dir.config_dir()); // Eat errors for breakfast
    dir
}

/// Main settings tree
#[derive(Default, Debug)]
pub struct Settings {
    base: PathBuf,
    app: RwLock<app::AppSettings>,
    clients: RwLock<clients::ClientSettings>,
    cards: RwLock<cards::CardSettings>,
}

impl Settings {
    /// Create a new settings tree from a config path
    pub fn init<'p>(path: impl Into<&'p Path>) -> Result<Arc<Settings>, SettingsError> {
        let base = path.into().to_path_buf();

        let this = Arc::new(Self {
            app: RwLock::new(load_path(base.join("app.json"))),
            clients: RwLock::new(load_path(base.join("clients.json"))),
            cards: RwLock::new(load_path(base.join("cards.json"))),
            base,
        });
        this.sync()?;
        Ok(this)
    }

    /// Sync any changes back to disk
    pub fn sync(self: &Arc<Self>) -> Result<(), SettingsError> {
        vec![
            ("app.json", serde_json::to_string_pretty(&self.app)?),
            ("clients.json", serde_json::to_string_pretty(&self.clients)?),
            ("cards.json", serde_json::to_string_pretty(&self.cards)?),
        ]
        .into_iter()
        .map(|(path, json)| {
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(self.base.join(path))
                .and_then(|mut f| f.write_all(json.as_bytes()))
                .map_err(Into::into)
        })
        .collect::<Result<Vec<_>, SettingsError>>()
        .map(|_| ())
    }

    /// Get read access to any stored setting
    pub fn r<'this>(self: &'this Arc<Self>) -> ReadSettings<'this> {
        ReadSettings { inner: self }
    }

    /// Wait to get exclusive write access to any settings
    ///
    /// This function can only be called from the `model` tree to
    /// avoid having the UI randomly change settings.  All settings
    /// changes _must_ be performed in the model
    pub(in crate::model) fn w<'this>(self: &'this Arc<Self>) -> WriteSettings<'this> {
        WriteSettings { inner: self }
    }
}

fn load_path<T: Default + DeserializeOwned>(path: PathBuf) -> T {
    File::open(path)
        .and_then(|mut f| {
            let mut c = String::new();
            f.read_to_string(&mut c).map(|_| c)
        })
        .and_then(|s| serde_json::from_str(&s).map_err(Into::into))
        .unwrap_or_else(|_| T::default())
}

pub struct ReadSettings<'settings> {
    inner: &'settings Arc<Settings>,
}

impl<'s> ReadSettings<'s> {
    /// Get read access to the `app` settings
    pub fn app(self) -> RwLockReadGuard<'s, app::AppSettings> {
        self.inner.app.read().unwrap()
    }

    /// Get read access to the `clients` settings
    pub fn clients(self) -> RwLockReadGuard<'s, clients::ClientSettings> {
        self.inner.clients.read().unwrap()
    }

    /// Get read access to the `cards` settings
    pub fn cards(self) -> RwLockReadGuard<'s, cards::CardSettings> {
        self.inner.cards.read().unwrap()
    }
}

pub struct WriteSettings<'settings> {
    inner: &'settings Arc<Settings>,
}

impl<'s> WriteSettings<'s> {
    /// Get read access to the `app` settings
    pub fn app(self) -> RwLockWriteGuard<'s, app::AppSettings> {
        self.inner.app.write().unwrap()
    }

    /// Get read access to the `clients` settings
    pub fn clients(self) -> RwLockWriteGuard<'s, clients::ClientSettings> {
        self.inner.clients.write().unwrap()
    }

    /// Get read access to the `cards` settings
    pub fn cards(self) -> RwLockWriteGuard<'s, cards::CardSettings> {
        self.inner.cards.write().unwrap()
    }
}
