mod application;
mod frequency_widget;
mod harmonics;
mod peak_sqrt_widget;
mod rms_trend;
mod rms_widget;
mod settings;
mod time;
mod time_chart;

use application::{calculate_precision, Application, VoltageUnit};
use conductor::{core::pipeline::Pipeline, prelude::*};
use core::f64;
use egui::ViewportBuilder;
use egui_plot::CoordinatesFormatter;
use frequency_widget::frequency_widget;
use harmonics::harmonics;
use peak_sqrt_widget::peak_sqrt;
use rms_trend::rms_trend;
use settings::{Settings, SettingsPacket};
use std::{
    sync::{
        mpsc::{channel, Receiver},
        Arc, RwLock,
    },
    thread,
};
use time_chart::time_chart;

const DARK_GRAY: egui::Color32 = egui::Color32::from_rgb(60, 60, 60);

pub fn coordinates_formatter<'a>(unit: VoltageUnit, precision: usize) -> CoordinatesFormatter<'a> {
    CoordinatesFormatter::new(move |plot_point, bounds| {
        let x_precision = precision.max(calculate_precision(&(bounds.min()[0]..=bounds.max()[0])));
        let y_precision = precision.max(calculate_precision(&(bounds.min()[1]..=bounds.max()[1])));

        let x = plot_point.x;
        let y = unit.apply_unit_with_precision(plot_point.y, y_precision);

        format!(
            "x = {:.precision$} s\ny = {}",
            x,
            y,
            precision = x_precision
        )
    })
}

#[derive(Clone, Copy)]
struct PeakVoltmeterPacket(i32);

impl From<PeakVoltmeterPacket> for f32 {
    fn from(packet: PeakVoltmeterPacket) -> Self {
        packet.0 as f32
    }
}

impl UdpDeserializer for PeakVoltmeterPacket {
    fn max_packet_size() -> usize {
        size_of::<i32>()
    }

    fn deserialize_packet(bytes: &[u8]) -> Self {
        // Self(i32::from_be_bytes([bytes[5], bytes[6], bytes[7], 0]))
        Self(i32::from_ne_bytes(bytes.try_into().unwrap()))
    }
}

fn create_pipeline(
    time_chart_buffer: Arc<RwLock<Vec<[f64; 2]>>>,
    harmonics_buffer: Arc<RwLock<Vec<[f64; 2]>>>,
    rms_trend_buffer: Arc<RwLock<Vec<[f64; 2]>>>,
    peak_sqrt_buffer: Arc<RwLock<Vec<[f64; 2]>>>,
    frequency_widget_buffer: Arc<RwLock<Vec<[f64; 2]>>>,
    receiver: Receiver<SettingsPacket>,
) -> Pipeline<(), ()> {
    let settings = Settings::new(receiver);

    let udp_receiver = UdpReceiver::<PeakVoltmeterPacket>::new("127.0.0.1:8080");

    let into_f32 = Intoer::<_, f32>::new();

    let calibration_factor_multiplier = Multiplier::new();

    let hv_factor_divider = Divider::new();

    let time_chart = time_chart(time_chart_buffer);
    let harmonics = harmonics(harmonics_buffer);
    let rms_trend = rms_trend(rms_trend_buffer);
    let peak_sqrt = peak_sqrt(peak_sqrt_buffer);
    let frequency_widget = frequency_widget(frequency_widget_buffer);

    settings.sample_rate.connect(&time_chart.input.sample_rate);
    settings.sample_rate.connect(&harmonics.input.sample_rate.0);
    settings.sample_rate.connect(&harmonics.input.sample_rate.1);
    settings.sample_rate.connect(&rms_trend.input.sample_rate.0);
    settings.sample_rate.connect(&rms_trend.input.sample_rate.1);
    settings
        .sample_rate
        .connect(&frequency_widget.input.sample_rate);

    settings
        .time_chart_periods
        .connect(&time_chart.input.periods);

    settings
        .adc_calibration_factor
        .connect(&calibration_factor_multiplier.input2);

    settings
        .hv_divider_factor
        .connect(&hv_factor_divider.input2);

    settings.fft_size.connect(&harmonics.input.fft_size.0);
    settings.fft_size.connect(&harmonics.input.fft_size.1);
    settings.fft_size.connect(&frequency_widget.input.fft_size);

    settings
        .harmonics_refresh_period
        .connect(&harmonics.input.refresh_period);
    settings
        .harmonics_refresh_period
        .connect(&frequency_widget.input.refresh_period);

    settings.window.connect(&rms_trend.input.window);

    settings.chart_size.connect(&rms_trend.input.chart_size);
    settings.chart_size.connect(&peak_sqrt.input.chart_size);
    settings
        .chart_size
        .connect(&frequency_widget.input.chart_size);

    settings
        .rms_refresh_period
        .connect(&rms_trend.input.refresh_preiod.0);
    settings
        .rms_refresh_period
        .connect(&rms_trend.input.refresh_preiod.1);
    settings
        .rms_refresh_period
        .connect(&peak_sqrt.input.refresh_period);

    udp_receiver.output.connect(&into_f32.input);

    into_f32
        .output
        .connect(&calibration_factor_multiplier.input1);

    calibration_factor_multiplier
        .output
        .connect(&hv_factor_divider.input1);

    hv_factor_divider.output.connect(&time_chart.input.data.0);
    hv_factor_divider.output.connect(&time_chart.input.data.1);
    hv_factor_divider.output.connect(&harmonics.input.data);
    hv_factor_divider.output.connect(&rms_trend.input.data);

    harmonics
        .output
        .fft_output
        .connect(&frequency_widget.input.fft_input);
    rms_trend
        .output
        .windowed_downsampled_data
        .connect(&peak_sqrt.input.windowed_downsampled_data);

    pipeline!(
        settings,
        udp_receiver,
        into_f32,
        calibration_factor_multiplier,
        hv_factor_divider,
        time_chart,
        harmonics,
        rms_trend,
        peak_sqrt,
        frequency_widget
    )
}

fn main() {
    let time_chart_buffer = Arc::new(RwLock::new(Vec::new()));
    let harmonics_buffer = Arc::new(RwLock::new(Vec::new()));
    let rms_trend_buffer = Arc::new(RwLock::new(Vec::new()));
    let peak_sqrt_buffer = Arc::new(RwLock::new(Vec::new()));
    let frequency_widget_buffer = Arc::new(RwLock::new(Vec::new()));

    let (sender, receiver) = channel();

    let time_chart_buffer_cloned = time_chart_buffer.clone();
    let harmonics_buffer_cloned = harmonics_buffer.clone();
    let rms_trend_buffer_cloned = rms_trend_buffer.clone();
    let peak_sqrt_buffer_cloned = peak_sqrt_buffer.clone();
    let frequency_widget_buffer_cloned = frequency_widget_buffer.clone();

    thread::spawn(move || {
        create_pipeline(
            time_chart_buffer_cloned,
            harmonics_buffer_cloned,
            rms_trend_buffer_cloned,
            peak_sqrt_buffer_cloned,
            frequency_widget_buffer_cloned,
            receiver,
        )
        .run();
    });

    let viewport = ViewportBuilder::default().with_fullscreen(true);

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Plotter",
        options,
        Box::new(|_cc| {
            Ok(Box::new(Application::new(
                time_chart_buffer,
                harmonics_buffer,
                rms_trend_buffer,
                peak_sqrt_buffer,
                frequency_widget_buffer,
                sender,
            )))
        }),
    )
    .unwrap();
}
