use crate::{
    frequency_widget::FrequencyWidget,
    harmonics::Harmonics,
    peak_sqrt_widget::PeakSqrtChart,
    rms_trend::RmsTrend,
    rms_widget::RmsWidget,
    settings::{
        CalibrationFactor, ChartSize, FftSize, RefreshPeriod, RmsWindow, SettingsPacket,
        TimeChartPeriods,
    },
    time::Time,
    time_chart::TimeChart,
};
use core::fmt;
use egui::{Align, Layout, RichText, Style, Visuals};
use std::{
    fmt::{Display, Formatter},
    ops::RangeInclusive,
    sync::{mpsc::Sender, Arc, RwLock},
};

pub fn calculate_precision(range: &RangeInclusive<f64>) -> usize {
    let max_abs = range.start().abs().max(range.end().abs());

    if max_abs >= 10.0 {
        0
    } else if max_abs >= 1.0 {
        1
    } else {
        let mut precision = 2;
        let mut step = 0.1;

        while max_abs < step && precision < 15 {
            precision += 1;
            step /= 10.0;
        }

        precision
    }
}

pub type Precision = usize;

const SAMPLE_RATE_DEFAULT: usize = 3125;
const CALIBRATION_FACTOR_DEFAULT: CalibrationFactor = 0.00319929;
const DEFAULT_UNIT: VoltageUnit = VoltageUnit::Volt;
const DEFAULT_PRECISION: Precision = 2;
const PERIODS_DEFAULT: TimeChartPeriods = 3;
const CHART_X_BOUND_DEFAULT: usize = 187;
const FFT_SIZE_DEFAULT: FftSize = 2048;
const HARMONICS_REFRESH_PERIOD: RefreshPeriod = 0.2;
const WINDOW_DEFAULT: RmsWindow = 0.5;
const CHART_SIZE_DEFAULT: ChartSize = 180;
const RMS_REFRESH_PERIOD_DEFAULT: RefreshPeriod = 0.5;

pub const CHART_X_BOUND_MARGIN: usize = 1;

#[derive(PartialEq)]
enum Panel {
    Charts,
    Settings,
}

#[derive(PartialEq, Clone, Copy)]
pub enum VoltageUnit {
    Volt,
    KiloVolt,
}

impl Display for VoltageUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            VoltageUnit::Volt => write!(f, "Volt"),
            VoltageUnit::KiloVolt => write!(f, "Kilovolt"),
        }
    }
}

impl VoltageUnit {
    pub fn apply_unit_with_precision(&self, value: f64, precision: usize) -> String {
        match self {
            VoltageUnit::Volt => format!("{:.precision$} V", value, precision = precision),
            VoltageUnit::KiloVolt => {
                format!("{:.precision$} kV", value / 1000.0, precision = precision)
            }
        }
    }
}

pub struct Application {
    time_chart: TimeChart,
    harmonics: Harmonics,
    rms_trend: RmsTrend,
    time: Time,
    peak_sqrt_chart: PeakSqrtChart,
    rms_widget: RmsWidget,
    frequency_widget: FrequencyWidget,

    panel: Panel,

    settings_sender: Sender<SettingsPacket>,

    // signal settings
    sample_rate: usize,
    calibration_factor: CalibrationFactor,
    unit: VoltageUnit,
    precision: Precision,

    // general settings
    chart_size: ChartSize,

    // time chart settings
    periods: TimeChartPeriods,
    chart_x_bound: usize,

    // harmonics and frequency settings
    fft_size: FftSize,
    harmonics_refresh_period: RefreshPeriod,

    // rms trend and peak sqrt settings
    window: RmsWindow,
    rms_refresh_period: RefreshPeriod,
}

impl Application {
    pub fn new(
        time_chart_data: Arc<RwLock<Vec<[f64; 2]>>>,
        harmonics_data: Arc<RwLock<Vec<[f64; 2]>>>,
        rms_trend_data: Arc<RwLock<Vec<[f64; 2]>>>,
        peak_sqrt_data: Arc<RwLock<Vec<[f64; 2]>>>,
        frequency_widget_data: Arc<RwLock<Vec<[f64; 2]>>>,
        settings_sender: Sender<SettingsPacket>,
    ) -> Self {
        // Set default settings
        settings_sender
            .send(SettingsPacket::SampleRate(SAMPLE_RATE_DEFAULT as f32))
            .unwrap();
        settings_sender
            .send(SettingsPacket::CalibrationFactor(
                CALIBRATION_FACTOR_DEFAULT,
            ))
            .unwrap();
        settings_sender
            .send(SettingsPacket::ChartSize(CHART_SIZE_DEFAULT))
            .unwrap();
        settings_sender
            .send(SettingsPacket::TimeChartPeriods(PERIODS_DEFAULT))
            .unwrap();
        settings_sender
            .send(SettingsPacket::FftSize(FFT_SIZE_DEFAULT))
            .unwrap();
        settings_sender
            .send(SettingsPacket::HarmonicsRefreshPeriod(
                HARMONICS_REFRESH_PERIOD,
            ))
            .unwrap();
        settings_sender
            .send(SettingsPacket::Window(WINDOW_DEFAULT))
            .unwrap();
        settings_sender
            .send(SettingsPacket::RmsRefreshPeriod(RMS_REFRESH_PERIOD_DEFAULT))
            .unwrap();

        Self {
            time_chart: TimeChart::new(time_chart_data),
            harmonics: Harmonics::new(harmonics_data),
            rms_trend: RmsTrend::new(rms_trend_data.clone()),
            peak_sqrt_chart: PeakSqrtChart::new(peak_sqrt_data),
            rms_widget: RmsWidget::new(rms_trend_data),
            frequency_widget: FrequencyWidget::new(frequency_widget_data),
            time: Time::new(),
            panel: Panel::Charts,
            settings_sender,
            sample_rate: SAMPLE_RATE_DEFAULT,
            calibration_factor: CALIBRATION_FACTOR_DEFAULT,
            unit: DEFAULT_UNIT,
            precision: DEFAULT_PRECISION,
            chart_size: CHART_SIZE_DEFAULT,
            periods: PERIODS_DEFAULT,
            chart_x_bound: CHART_X_BOUND_DEFAULT,
            fft_size: FFT_SIZE_DEFAULT,
            harmonics_refresh_period: HARMONICS_REFRESH_PERIOD,
            window: WINDOW_DEFAULT,
            rms_refresh_period: RMS_REFRESH_PERIOD_DEFAULT,
        }
    }

    fn charts(&mut self, ctx: &egui::Context) {
        let available_size = ctx.available_rect().size();

        egui::SidePanel::right("side_panel")
            .resizable(false)
            .exact_width(available_size.x / 5.0)
            .show(ctx, |ui| {
                self.time.ui(ui);

                ui.separator();

                self.peak_sqrt_chart
                    .ui(ui, self.chart_size, self.unit, self.precision);

                self.rms_widget
                    .ui(ui, self.chart_size, self.unit, self.precision);

                self.frequency_widget
                    .ui(ui, self.chart_size, self.precision);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                self.time_chart.ui(
                    ui,
                    self.chart_x_bound,
                    self.sample_rate as f32,
                    self.unit,
                    self.precision,
                );

                ui.separator();

                self.harmonics
                    .ui(ui, self.sample_rate as f32, self.precision);

                ui.separator();

                self.rms_trend
                    .ui(ui, self.chart_size, self.unit, self.precision);
            });

            ui.ctx().request_repaint();
        });
    }

    fn settings(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;

            ui.label(RichText::new("Signal Settings").size(20.0).strong());

            ui.horizontal(|ui| {
                ui.label("Signal Sample Rate:");
                if ui
                    .add(egui::Slider::new(&mut self.sample_rate, 1..=10_000).text("Hz"))
                    .changed()
                {
                    self.settings_sender
                        .send(SettingsPacket::SampleRate(self.sample_rate as f32))
                        .unwrap();
                }
            });

            ui.horizontal(|ui| {
                ui.label("Calibration Factor:");
                if ui
                    .add(egui::Slider::new(&mut self.calibration_factor, 0.0..=1.0))
                    .changed()
                {
                    self.settings_sender
                        .send(SettingsPacket::CalibrationFactor(self.calibration_factor))
                        .unwrap();
                }
            });

            egui::ComboBox::from_label("Volatage Unit")
                .selected_text(format!("{}", self.unit))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.unit, VoltageUnit::Volt, "Volt");
                    ui.selectable_value(&mut self.unit, VoltageUnit::KiloVolt, "Kilovolt");
                });

            ui.horizontal(|ui| {
                ui.label("Chart Formatter Precision:");
                ui.add(egui::Slider::new(&mut self.precision, 0..=10))
            });

            ui.separator();

            ui.label(RichText::new("General Settings").size(20.0).strong());

            ui.horizontal(|ui| {
                ui.label("Chart Size:");
                if ui
                    .add(egui::Slider::new(&mut self.chart_size, 10..=300).text("seconds"))
                    .changed()
                {
                    self.settings_sender
                        .send(SettingsPacket::ChartSize(self.chart_size))
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

            ui.label(
                RichText::new("Harmonics and Frequency Settings")
                    .size(20.0)
                    .strong(),
            );

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

            ui.horizontal(|ui| {
                ui.label("Refresh Period:");
                if ui
                    .add(
                        egui::Slider::new(&mut self.harmonics_refresh_period, 0.01..=10.0)
                            .text("seconds"),
                    )
                    .changed()
                {
                    self.settings_sender
                        .send(SettingsPacket::HarmonicsRefreshPeriod(
                            self.harmonics_refresh_period,
                        ))
                        .unwrap();
                }
            });

            ui.separator();

            ui.label(
                RichText::new("RMS Trend and Peak Sqrt Settings")
                    .size(20.0)
                    .strong(),
            );

            ui.horizontal(|ui| {
                ui.label("Window Size:");
                if ui
                    .add(egui::Slider::new(&mut self.window, 0.01..=12.0).text("seconds"))
                    .changed()
                {
                    self.settings_sender
                        .send(SettingsPacket::Window(self.window))
                        .unwrap();
                }
            });

            ui.horizontal(|ui| {
                ui.label("Refresh Period:");
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
                ui.selectable_value(&mut self.panel, Panel::Charts, "Charts");
                ui.selectable_value(&mut self.panel, Panel::Settings, "Settings");

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
