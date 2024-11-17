use chrono::{Datelike, Local};
use egui::RichText;

pub struct Time {}

impl Time {
    pub fn new() -> Self {
        Self {}
    }

    fn current_time() -> String {
        Local::now().format("%H:%M:%S").to_string()
    }

    fn current_date() -> String {
        Local::now().format("%Y-%m-%d").to_string()
    }

    fn current_day() -> String {
        match Local::now().weekday() {
            chrono::Weekday::Mon => "Mon",
            chrono::Weekday::Tue => "Tue",
            chrono::Weekday::Wed => "Wed",
            chrono::Weekday::Thu => "Thu",
            chrono::Weekday::Fri => "Fri",
            chrono::Weekday::Sat => "Sat",
            chrono::Weekday::Sun => "Sun",
        }
        .to_string()
    }

    pub fn ui(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(Self::current_day()).size(20.0).strong());

            ui.label(RichText::new(Self::current_time()).size(40.0).strong());

            ui.label(RichText::new(Self::current_date()).size(20.0).strong());
        });
    }
}
