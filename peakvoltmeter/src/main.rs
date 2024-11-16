mod trigger;
mod ui;

use conductor::prelude::*;
use core::f64;
use egui::ViewportBuilder;
use rustfft::num_complex::Complex;
use std::{
    sync::{Arc, RwLock},
    thread,
};
use trigger::RisingEdgeTrigger;
use ui::{harmonics::Harmonics, rms_trend::RmsTrend, time_chart::TimeChart, Application};

pub const FFT_SIZE: usize = 2048; // samples
pub const SAMPLE_RATE: usize = 3125; // samples per second
pub const RMS_WINDOW: f32 = 0.5; // seconds
pub const RMS_CHART_SIZE: f32 = 180.0; // seconds

fn main() {
    let time_chart_buffer = Arc::new(RwLock::new(Vec::new()));
    let harmonics_buffer = Arc::new(RwLock::new(Vec::new()));
    let rms_trend_buffer = Arc::new(RwLock::new(Vec::new()));

    let udp_receiver = UdpReceiver::<PeakVoltmeterPacket>::new("127.0.0.1:8080");
    let into_i32 = Intoer::<_, i32>::new();
    let into_f32 = Intoer::<_, f32>::new();

    let trigger = RisingEdgeTrigger::new(0);
    let period = Downsampler::new(3);

    let fft_buffer = Buffer::new(false);
    fft_buffer.size.set_initial(FFT_SIZE);

    let hann_window = Window::new(WindowType::Hann);

    let fft = FFT::new();
    let downsampler = Downsampler::new(600);
    let lambda = Lambdaer::new(|fft: Vec<Complex<f32>>| {
        let length = fft.len();

        let fft = fft
            .into_iter()
            .take(length / 2)
            .map(|value| value.norm() as f64)
            .collect::<Vec<_>>();

        let max = fft.iter().cloned().fold(f64::MIN, f64::max);

        fft.into_iter()
            .map(|value| 20.0 * (value / max).log10())
            .collect()
    });

    let time_chart = TimeChart::new(time_chart_buffer.clone());
    let harmonics = Harmonics::new(harmonics_buffer.clone());
    let rms_trend = RmsTrend::new(rms_trend_buffer.clone());

    udp_receiver.output.connect(&into_i32.input);
    udp_receiver.output.connect(&into_f32.input);

    trigger.trigger.connect(&period.input);
    period.output.connect(&time_chart.trigger);
    into_i32.output.connect(&trigger.input);
    into_i32.output.connect(&time_chart.input);

    into_f32.output.connect(&fft_buffer.input);
    fft_buffer.output.connect(&downsampler.input);
    downsampler.output.connect(&hann_window.input);
    hann_window.output.connect(&fft.input);
    fft.output.connect(&lambda.input);
    lambda.output.connect(&harmonics.input);

    into_i32.output.connect(&rms_trend.input);

    thread::spawn(move || {
        pipeline!(
            udp_receiver,
            into_i32,
            into_f32,
            trigger,
            fft_buffer,
            hann_window,
            period,
            fft,
            downsampler,
            lambda,
            time_chart,
            harmonics,
            rms_trend
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
            )))
        }),
    )
    .unwrap();
}

#[derive(Clone, Copy)]
struct PeakVoltmeterPacket(i32);

impl From<PeakVoltmeterPacket> for i32 {
    fn from(packet: PeakVoltmeterPacket) -> Self {
        packet.0
    }
}

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
