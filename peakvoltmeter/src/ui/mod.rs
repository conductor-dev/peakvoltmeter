pub mod time_chart;

use egui::{Style, Visuals};
use std::sync::{Arc, RwLock};
use time_chart::TimeChartUi;

pub(crate) struct Application {
    time_chart: TimeChartUi,
}

impl Application {
    pub(crate) fn new(data: Arc<RwLock<Vec<i32>>>) -> Self {
        Self {
            time_chart: TimeChartUi::new(data),
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let style = Style {
            visuals: Visuals::dark(),
            ..Style::default()
        };
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            self.time_chart.ui(ui);

            ui.ctx().request_repaint();
        });
    }
}
