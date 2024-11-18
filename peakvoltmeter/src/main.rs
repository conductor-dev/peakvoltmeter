mod application;
mod harmonics;
mod peak_sqrt;
mod rms_trend;
mod settings;
mod time;
mod time_chart;

use application::Application;
use conductor::{core::pipeline::Pipeline, prelude::*};
use core::f64;
// use egui::ViewportBuilder;
use harmonics::harmonics;
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

fn create_pipeline(
    time_chart_buffer: Arc<RwLock<Vec<f64>>>,
    harmonics_buffer: Arc<RwLock<Vec<f64>>>,
    rms_trend_buffer: Arc<RwLock<Vec<f64>>>,
    receiver: Receiver<SettingsPacket>,
) -> Pipeline<(), ()> {
    let settings = Settings::new(receiver);

    let udp_receiver = UdpReceiver::<PeakVoltmeterPacket>::new("127.0.0.1:8080");

    let time_chart = time_chart(time_chart_buffer);
    let harmonics = harmonics(harmonics_buffer);
    let rms_trend = rms_trend(rms_trend_buffer);

    settings.sample_rate.connect(&rms_trend.input.sample_rate);
    settings
        .time_chart_periods
        .connect(&time_chart.input.periods);
    settings.fft_size.connect(&harmonics.input.fft_size);
    settings.rms_window.connect(&rms_trend.input.rms_window);
    settings
        .rms_chart_size
        .connect(&rms_trend.input.rms_chart_size);
    settings
        .rms_refresh_period
        .connect(&rms_trend.input.rms_refresh_period);

    udp_receiver.output.connect(&time_chart.input.data);
    udp_receiver.output.connect(&harmonics.input.data);
    udp_receiver.output.connect(&rms_trend.input.data);

    pipeline!(settings, udp_receiver, time_chart, harmonics, rms_trend)
}

fn main() {
    let time_chart_buffer = Arc::new(RwLock::new(Vec::new()));
    let harmonics_buffer = Arc::new(RwLock::new(Vec::new()));
    let rms_trend_buffer = Arc::new(RwLock::new(Vec::new()));
    let peak_sqrt_buffer = Arc::new(RwLock::new(Vec::new()));

    let (sender, receiver) = channel();

    let time_chart_buffer_cloned = time_chart_buffer.clone();
    let harmonics_buffer_cloned = harmonics_buffer.clone();
    let rms_trend_buffer_cloned = rms_trend_buffer.clone();

    thread::spawn(move || {
        create_pipeline(
            time_chart_buffer_cloned,
            harmonics_buffer_cloned,
            rms_trend_buffer_cloned,
            receiver,
        )
        .run();
    });

    // let viewport = ViewportBuilder::default().with_fullscreen(true);

    let options = eframe::NativeOptions {
        // viewport,
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
                sender,
            )))
        }),
    )
    .unwrap();
}
