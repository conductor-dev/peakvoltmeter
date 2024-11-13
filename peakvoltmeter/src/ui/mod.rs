pub mod harmonics;
pub mod rms_trend;
pub mod time_chart;

use egui::{Style, Visuals};
use harmonics::HarmonicsUi;
use rms_trend::RmsTrendUi;
use std::sync::{Arc, RwLock};
use time_chart::TimeChartUi;

pub(crate) struct Application {
    time_chart: TimeChartUi,
    harmonics: HarmonicsUi,
    rms_trend: RmsTrendUi,
}

impl Application {
    pub(crate) fn new(
        time_chart_data: Arc<RwLock<Vec<f64>>>,
        harmonics_data: Arc<RwLock<Vec<f64>>>,
        rms_trend_data: Arc<RwLock<Vec<f64>>>,
    ) -> Self {
        Self {
            time_chart: TimeChartUi::new(time_chart_data),
            harmonics: HarmonicsUi::new(harmonics_data),
            rms_trend: RmsTrendUi::new(rms_trend_data),
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

        egui::SidePanel::right("side_panel").show(ctx, |ui| {
            ui.label("Peak Voltmeterrrrrrrrrrrrrrrrrrrrrr");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                self.time_chart.ui(ui);

                ui.separator();

                self.harmonics.ui(ui);

                ui.separator();

                self.rms_trend.ui(ui);
            });

            ui.ctx().request_repaint();
        });
    }
}
