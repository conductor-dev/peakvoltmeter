use crate::{harmonics::Harmonics, rms_trend::RmsTrend, time_chart::TimeChart};
use egui::{Style, Visuals};
use std::sync::{Arc, RwLock};

pub(crate) struct Application {
    time_chart: TimeChart,
    harmonics: Harmonics,
    rms_trend: RmsTrend,
}

impl Application {
    pub(crate) fn new(
        time_chart_data: Arc<RwLock<Vec<f64>>>,
        harmonics_data: Arc<RwLock<Vec<f64>>>,
        rms_trend_data: Arc<RwLock<Vec<f64>>>,
    ) -> Self {
        Self {
            time_chart: TimeChart::new(time_chart_data),
            harmonics: Harmonics::new(harmonics_data),
            rms_trend: RmsTrend::new(rms_trend_data),
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
