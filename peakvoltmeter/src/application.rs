use crate::{
    harmonics::Harmonics,
    peak_sqrt::PeakSqrtChart,
    rms_trend::RmsTrend,
    settings::{
        FftSize, RmsChartSize, RmsRefreshPeriod, RmsWindow, SampleRate, SettingsPacket,
        TimeChartPeriods,
    },
    time::Time,
    time_chart::TimeChart,
};
use egui::{Align, Layout, RichText, Style, Visuals};
use std::sync::{mpsc::Sender, Arc, RwLock};

const SAMPLE_RATE_DEFAULT: SampleRate = 3125;
const PERIODS_DEFAULT: TimeChartPeriods = 3;
const CHART_X_BOUND_DEFAULT: usize = 185;
const FFT_SIZE_DEFAULT: FftSize = 2048;
const RMS_WINDOW_DEFAULT: RmsWindow = 0.5;
const RMS_CHART_SIZE_WINDOW_DEFAULT: usize = 180;
const RMS_REFRESH_PERIOD_DEFAULT: RmsRefreshPeriod = 0.5;

pub const CHART_X_BOUND_MARGIN: usize = 10;

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
    peak_sqrt_chart: PeakSqrtChart,

    panel: Panel,

    settings_sender: Sender<SettingsPacket>,

    // general settings
    pub sample_rate: SampleRate,

    // time chart settings
    pub periods: TimeChartPeriods,
    pub chart_x_bound: usize,

    // harmonics settings
    pub fft_size: FftSize,

    // rms trend settings
    pub rms_window: RmsWindow,
    pub rms_chart_size: RmsChartSize,
    pub rms_refresh_period: RmsRefreshPeriod,
}

impl Application {
    pub fn new(
        time_chart_data: Arc<RwLock<Vec<f64>>>,
        harmonics_data: Arc<RwLock<Vec<f64>>>,
        rms_trend_data: Arc<RwLock<Vec<f64>>>,
        peak_sqrt_data: Arc<RwLock<Vec<f64>>>,
        settings_sender: Sender<SettingsPacket>,
    ) -> Self {
        // Set default settings
        settings_sender
            .send(SettingsPacket::SampleRate(SAMPLE_RATE_DEFAULT))
            .unwrap();
        settings_sender
            .send(SettingsPacket::TimeChartPeriods(PERIODS_DEFAULT))
            .unwrap();
        settings_sender
            .send(SettingsPacket::FftSize(FFT_SIZE_DEFAULT))
            .unwrap();
        settings_sender
            .send(SettingsPacket::RmsWindow(RMS_WINDOW_DEFAULT))
            .unwrap();
        settings_sender
            .send(SettingsPacket::RmsChartSize(RMS_CHART_SIZE_WINDOW_DEFAULT))
            .unwrap();
        settings_sender
            .send(SettingsPacket::RmsRefreshPeriod(RMS_REFRESH_PERIOD_DEFAULT))
            .unwrap();

        Self {
            time_chart: TimeChart::new(time_chart_data),
            harmonics: Harmonics::new(harmonics_data),
            rms_trend: RmsTrend::new(rms_trend_data),
            peak_sqrt_chart: PeakSqrtChart::new(peak_sqrt_data),
            time: Time::new(),
            panel: Panel::Charts,
            settings_sender,
            sample_rate: SAMPLE_RATE_DEFAULT,
            periods: PERIODS_DEFAULT,
            chart_x_bound: CHART_X_BOUND_DEFAULT,
            fft_size: FFT_SIZE_DEFAULT,
            rms_window: RMS_WINDOW_DEFAULT,
            rms_chart_size: RMS_CHART_SIZE_WINDOW_DEFAULT,
            rms_refresh_period: RMS_REFRESH_PERIOD_DEFAULT,
        }
    }

    fn charts(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("side_panel").show(ctx, |ui| {
            self.time.ui(ui);

            ui.separator();

            self.peak_sqrt_chart.ui(ui, self);
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
                ui.label("Signal Sample Rate:");
                if ui
                    .add(egui::Slider::new(&mut self.sample_rate, 1..=10000).text("Hz"))
                    .changed()
                {
                    self.settings_sender
                        .send(SettingsPacket::SampleRate(self.sample_rate))
                        .unwrap();
                }
            });

            ui.separator();

            ui.label(RichText::new("Time Chart Settings").size(20.0).strong());

            ui.horizontal(|ui| {
                ui.label("Periods:");
                if ui
                    .add(egui::Slider::new(&mut self.periods, 1..=10))
                    .changed()
                {
                    self.settings_sender
                        .send(SettingsPacket::TimeChartPeriods(self.periods))
                        .unwrap();
                }
            });

            ui.separator();

            ui.label(RichText::new("Harmonics Settings").size(20.0).strong());

            ui.horizontal(|ui| {
                ui.label("FFT Size:");
                if ui
                    .add(egui::Slider::new(&mut self.fft_size, 128..=8192).text("samples"))
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
                    .add(egui::Slider::new(&mut self.rms_window, 0.01..=12.0).text("seconds"))
                    .changed()
                {
                    self.settings_sender
                        .send(SettingsPacket::RmsWindow(self.rms_window))
                        .unwrap();
                }
            });

            ui.horizontal(|ui| {
                ui.label("RMS Chart Size:");
                if ui
                    .add(egui::Slider::new(&mut self.rms_chart_size, 10..=300).text("seconds"))
                    .changed()
                {
                    self.settings_sender
                        .send(SettingsPacket::RmsChartSize(self.rms_chart_size))
                        .unwrap();
                }
            });

            ui.horizontal(|ui| {
                ui.label("RMS Refresh Period:");
                if ui
                    .add(
                        egui::Slider::new(&mut self.rms_refresh_period, 0.01..=10.0)
                            .text("seconds"),
                    )
                    .changed()
                {
                    self.settings_sender
                        .send(SettingsPacket::RmsRefreshPeriod(self.rms_refresh_period))
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
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.selectable_value(&mut self.panel, Panel::Charts, "Charts");
                    ui.selectable_value(&mut self.panel, Panel::Settings, "Settings");
                });

                ui.add_space(ui.available_width());

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Reset Time Chart Bounds").clicked() {
                        self.chart_x_bound = self.time_chart.data.read().unwrap().len();
                    }
                });
            });
            ui.add_space(3.0);
        });

        match self.panel {
            Panel::Charts => self.charts(ctx),
            Panel::Settings => self.settings(ctx),
        };
    }
}
