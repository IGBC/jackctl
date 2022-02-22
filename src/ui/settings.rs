use super::utils;
use gtk::prelude::*;
use gtk::{Adjustment, Builder, Button, Label, Switch, Window};
use std::sync::Arc;

use crate::model::events::{UiEvent, UiSettingsUpdate};
use crate::model::settings::Settings;
use crate::ui::UiRuntime;

pub(super) struct SettingsWindow {
    window: Window,
    settings: Arc<Settings>,

    period_size: Adjustment,
    sample_rate: Adjustment,
    n_periods: Adjustment,
    resample_q: Adjustment,
    realtime_button: Switch,
    latency_view: Label,
}

const DP_OVERSAMPLE: f64 = 100.0;

impl SettingsWindow {
    pub fn new(settings: Arc<Settings>, runtime: UiRuntime) -> Arc<Self> {
        let builder = Builder::from_resource("/net/jackctl/Jackctl/jack_settings.glade");
        let window = utils::get_object(&builder, "settingsDialog");

        let period_size: Adjustment = utils::get_object(&builder, "jackBlockSize");
        let sample_rate = utils::get_object(&builder, "jackSampleRate");
        let n_periods = utils::get_object(&builder, "jackPeriods");
        let resample_q = utils::get_object(&builder, "jackResampleQ");
        let realtime_button = utils::get_object(&builder, "jackSettingsRealtime");
        let latency_view = utils::get_object(&builder, "jackSettingsLatencyDisplay");

        let save: Button = utils::get_object(&builder, "settingsSave");

        let this = Arc::new(SettingsWindow {
            window,
            settings,

            period_size,
            sample_rate,
            n_periods,
            resample_q,
            realtime_button,
            latency_view,
        });

        let this_clone = this.clone();
        this.period_size
            .connect_value_changed(move |_| this_clone.update_latency());
        let this_clone = this.clone();
        this.sample_rate
            .connect_value_changed(move |_| this_clone.update_latency());
        let this_clone = this.clone();
        this.n_periods
            .connect_value_changed(move |_| this_clone.update_latency());

        let settings_window = this.clone();
        save.connect_clicked(move |_| {
            info!("Saving Settings");
            settings_window.window.hide();

            let period_size = settings_window.period_size.get_value().round() as u32;
            let sample_rate = settings_window.sample_rate.get_value().round() as u32;
            let n_periods = settings_window.n_periods.get_value().round() as u32;
            let realtime = settings_window.realtime_button.get_active();
            let resample_q = settings_window.resample_q.get_value().round() as u32;

            let event = UiEvent::UpdateSettings(UiSettingsUpdate {
                period_size,
                sample_rate,
                n_periods,
                realtime,
                resample_q,
            });

            runtime.sender().send(event);
        });

        this
    }

    pub fn show(&self) {
        let app_settings = self.settings.r().app();

        let jack_settings = &app_settings.jack;
        jack_settings.sample_rate;

        // update settings menu from file
        self.period_size.set_value(jack_settings.period_size as f64);
        self.sample_rate.set_value(jack_settings.sample_rate as f64);
        self.n_periods.set_value(jack_settings.n_periods as f64);
        self.resample_q.set_value(jack_settings.resample_q as f64);
        self.realtime_button.set_active(jack_settings.realtime);

        self.update_latency();

        self.window.show_all();
    }

    pub fn update_latency(&self) {
        let p = self.period_size.get_value();
        let sr = self.sample_rate.get_value();
        let n = self.n_periods.get_value();

        let latency_s = (p * n) / sr;
        let latency_ms = ((latency_s * 1000.0 * DP_OVERSAMPLE).round()) / DP_OVERSAMPLE;

        self.latency_view.set_markup(&format!("{}ms", latency_ms));
    }
}
