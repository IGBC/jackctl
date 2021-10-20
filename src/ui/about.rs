use super::get_object;
use gtk::{AboutDialog, AboutDialogExt, Builder, ButtonExt, DialogExt, ModelButton, WidgetExt};

pub struct About {
    inner: AboutDialog,
}

impl About {
    /// Create the main about dialog builder
    pub fn new(b: &Builder) -> Self {
        let inner: AboutDialog = get_object(b, "aboutdialog");
        inner.set_version(Some(env!("CARGO_PKG_VERSION")));
        inner.connect_response(move |dialog, _| dialog.hide());
        Self { inner }
    }

    /// Initialise a button to open the about dialog
    pub fn button(self, b: &Builder) {
        let button: ModelButton = get_object(b, "about.mainmenu");
        button.connect_clicked(move |_| self.inner.show());
    }
}
