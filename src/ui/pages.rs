use crate::ui::utils;
use glib::object::IsA;
use gtk::{prelude::*, Builder, Label, Notebook, PolicyType, Widget};
use std::collections::BTreeMap;

pub struct Pages {
    inner: Notebook,
    order: BTreeMap<String, u32>,
}

impl Pages {
    /// Initialise the page notebook with a list of page labels
    pub fn new(b: &Builder, pages: Vec<&str>) -> Self {
        let inner: Notebook = utils::get_object(b, "tabs.maindialog");
        inner.set_show_border(false);
        Self {
            inner,
            order: pages
                .into_iter()
                .enumerate()
                .map(|(pos, label)| (label.to_owned(), pos as u32))
                .collect(),
        }
    }

    fn insert<T: IsA<Widget>>(&mut self, label: &str, child: &T) {
        let pos = self
            .order
            .get(label)
            .expect("Tried to insert an unknown page!");

        self.inner.insert_page(
            &utils::wrap_scroll(child),
            Some(&Label::new(Some(label))),
            Some(*pos),
        );
    }

    #[inline]
    pub fn insert_scrolled<T: IsA<Widget>>(&mut self, label: &str, child: &T) {
        self.insert(label, &utils::wrap_scroll(child));
    }

    pub fn insert_horizontal<T: IsA<Widget>>(&mut self, label: &str, child: &T) {
        let horizontal = utils::wrap_scroll(child);
        horizontal.set_policy(PolicyType::Automatic, PolicyType::Never);
        self.insert(label, &horizontal);
    }

    /// Remove a page by label
    pub fn remove_page(&mut self, label: &str) {
        let pos = self
            .order
            .get(label)
            .expect("Tried to remove an unknown page!");
        self.inner.remove_page(Some(*pos));
    }

    /// Gets the label of the current page
    pub fn get_current(&self) -> String {
        let curr = self
            .inner
            .get_current_page()
            .expect("No page currently selected");
        self.order
            .iter()
            .find(|(_, pos)| pos == &&curr)
            .expect("Selected page not found!")
            .0
            .clone()
    }

    /// Set the current page
    pub fn set_current<S: Into<String>>(&mut self, label: S) {
        let label = label.into();
        let pos = self
            .order
            .get(label.as_str())
            .expect("Tried switching to an unknown page!");
        self.inner.set_current_page(Some(*pos));
    }

    pub fn show_all(&mut self) {
        self.inner.show_all();
    }
}
