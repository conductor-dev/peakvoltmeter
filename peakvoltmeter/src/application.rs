use crate::{
    harmonics::Harmonics,
    rms_trend::RmsTrend,
    settings::{FftSize, RmsWindow, SampleRate, SettingsPacket},
    time::Time,
    time_chart::TimeChart,
};
use egui::{RichText, Style, Visuals};
use std::sync::{mpsc::Sender, Arc, RwLock};

const SAMPLE_RATE_DEFAULT: SampleRate = 3125;
const FFT_SIZE_DEFAULT: FftSize = 2048;
const RMS_WINDOW_DEFAULT: RmsWindow = 0.5;

#[derive(PartialEq)]
enum Panel {
    Charts,
    Settings,
}

pub struct Application {
    time_chart: TimeChart,
    harmonics: Harmonics,
    rms_trend: RmsTrend,
    time: Time,

    panel: Panel,

    settings_sender: Sender<SettingsPacket>,

    pub sample_rate: SampleRate,
    pub fft_size: FftSize,
    pub rms_window: f32,
}

impl Application {
    pub fn new(
        time_chart_data: Arc<RwLock<Vec<f64>>>,
        harmonics_data: Arc<RwLock<Vec<f64>>>,
        rms_trend_data: Arc<RwLock<Vec<f64>>>,
        settings_sender: Sender<SettingsPacket>,
    ) -> Self {
        // Set default settings
        settings_sender
            .send(SettingsPacket::SampleRate(SAMPLE_RATE_DEFAULT))
            .unwrap();
        settings_sender
            .send(SettingsPacket::FftSize(FFT_SIZE_DEFAULT))
            .unwrap();
        settings_sender
            .send(SettingsPacket::RmsWindow(RMS_WINDOW_DEFAULT))
            .unwrap();

        Self {
            time_chart: TimeChart::new(time_chart_data),
            harmonics: Harmonics::new(harmonics_data),
            rms_trend: RmsTrend::new(rms_trend_data),
            time: Time::new(),
            panel: Panel::Charts,
            settings_sender,
            sample_rate: SAMPLE_RATE_DEFAULT,
            fft_size: FFT_SIZE_DEFAULT,
            rms_window: RMS_WINDOW_DEFAULT,
        }
    }

    fn charts(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("side_panel").show(ctx, |ui| {
            self.time.ui(ui);

            ui.separator();
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                self.time_chart.ui(ui, self);

                ui.separator();

                self.harmonics.ui(ui, self);

                ui.separator();

                self.rms_trend.ui(ui, self);
            });

            ui.ctx().request_repaint();
        });
    }

    fn settings(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;

            ui.label(RichText::new("General Settings").size(20.0).strong());

            ui.horizontal(|ui| {
                ui.label("Sample Rate:");
                if ui
                    .add(egui::Slider::new(&mut self.sample_rate, 0..=10000).text("Hz"))
                    .changed()
                {
                    self.settings_sender
                        .send(SettingsPacket::SampleRate(self.sample_rate))
                        .unwrap();
                }
            });

            ui.separator();

            ui.label(RichText::new("Time Chart Settings").size(20.0).strong());

            ui.separator();

            ui.label(RichText::new("Harmonics Settings").size(20.0).strong());

            ui.horizontal(|ui| {
                ui.label("FFT Size:");
                if ui
                    .add(egui::Slider::new(&mut self.fft_size, 0..=8192).text("samples"))
                    .changed()
                {
                    self.settings_sender
                        .send(SettingsPacket::FftSize(self.fft_size))
                        .unwrap();
                }
            });

            ui.separator();

            ui.label(RichText::new("RMS Trend Settings").size(20.0).strong());

            ui.horizontal(|ui| {
                ui.label("RMS Window:");
                if ui
                    .add(egui::Slider::new(&mut self.rms_window, 0.0..=12.0).text("seconds"))
                    .changed()
                {
                    self.settings_sender
                        .send(SettingsPacket::RmsWindow(self.rms_window))
                        .unwrap();
                }
            });
        });
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let style = Style {
            visuals: Visuals::dark(),
            ..Style::default()
        };
        ctx.set_style(style);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.panel, Panel::Charts, "Charts");
                ui.selectable_value(&mut self.panel, Panel::Settings, "Settings");
            });
            ui.add_space(3.0);
        });

        match self.panel {
            Panel::Charts => self.charts(ctx),
            Panel::Settings => self.settings(ctx),
        };
    }
}
