mod application;
mod harmonics;
mod rms_trend;
mod time_chart;

use application::Application;
use conductor::{core::pipeline::Pipeline, prelude::*};
use core::f64;
use egui::ViewportBuilder;
use harmonics::harmonics;
use rms_trend::rms_trend;
use std::{
    sync::{Arc, RwLock},
    thread,
};
use time_chart::time_chart;

pub const FFT_SIZE: usize = 2048; // samples
pub const SAMPLE_RATE: usize = 3125; // samples per second
pub const RMS_WINDOW: f32 = 0.5; // seconds
pub const RMS_CHART_SIZE: f32 = 180.0; // seconds

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
) -> Pipeline<(), ()> {
    let udp_receiver = UdpReceiver::<PeakVoltmeterPacket>::new("127.0.0.1:8080");

    let time_chart = time_chart(time_chart_buffer);
    let harmonics = harmonics(harmonics_buffer);
    let rms_trend = rms_trend(rms_trend_buffer);

    udp_receiver.output.connect(&time_chart.input);
    udp_receiver.output.connect(&harmonics.input);
    udp_receiver.output.connect(&rms_trend.input);

    pipeline!(udp_receiver, time_chart, harmonics, rms_trend)
}

fn main() {
    let time_chart_buffer = Arc::new(RwLock::new(Vec::new()));
    let harmonics_buffer = Arc::new(RwLock::new(Vec::new()));
    let rms_trend_buffer = Arc::new(RwLock::new(Vec::new()));

    let time_chart_buffer_cloned = time_chart_buffer.clone();
    let harmonics_buffer_cloned = harmonics_buffer.clone();
    let rms_trend_buffer_cloned = rms_trend_buffer.clone();

    thread::spawn(move || {
        create_pipeline(
            time_chart_buffer_cloned,
            harmonics_buffer_cloned,
            rms_trend_buffer_cloned,
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
