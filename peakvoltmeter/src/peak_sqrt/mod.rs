use eframe::egui::Frame;
use egui::{Color32, Rounding};

use crate::application::Application;
use std::sync::{Arc, RwLock};

pub struct PeakSqrtChart {
    _data: Arc<RwLock<Vec<[f64; 2]>>>,
}

impl PeakSqrtChart {
    pub fn new(data: Arc<RwLock<Vec<[f64; 2]>>>) -> Self {
        Self { _data: data }
    }

    pub fn ui(&self, ui: &mut egui::Ui, _application: &Application) {
        let frame = Frame::default()
            .fill(Color32::LIGHT_GRAY)
            .rounding(Rounding::same(10.0));

        frame.show(ui, |ui| ui.label("Hallo"));
    }
}
