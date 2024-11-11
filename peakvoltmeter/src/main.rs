mod trigger;
mod ui;

use conductor::prelude::*;
use std::{
    sync::{Arc, RwLock},
    thread,
};
use trigger::RisingEdgeTrigger;
use ui::{time_chart::TimeChart, Application};

fn main() {
    let buffer = Arc::new(RwLock::new(Vec::new()));

    let udp_receiver = UdpReceiver::<PeakVoltmeterPacket>::new("127.0.0.1:8080");
    let into = Intoer::new();

    let trigger = RisingEdgeTrigger::new(0);
    let downsampler = Downsampler::new(3);

    let time_chart = TimeChart::new(buffer.clone());

    udp_receiver.output.connect(&into.input);
    into.output.connect(&time_chart.input);
    into.output.connect(&trigger.input);

    trigger.trigger.connect(&downsampler.input);
    downsampler.output.connect(&time_chart.trigger);

    thread::spawn(move || {
        pipeline!(udp_receiver, into, trigger, time_chart, downsampler).run();
    });

    eframe::run_native(
        "Plotter",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(Application::new(buffer)))),
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

impl UdpDeserializer for PeakVoltmeterPacket {
    fn max_packet_size() -> usize {
        size_of::<i32>()
    }

    fn deserialize_packet(bytes: &[u8]) -> Self {
        // Self(i32::from_be_bytes([bytes[5], bytes[6], bytes[7], 0]))
        Self(i32::from_ne_bytes(bytes.try_into().unwrap()))
    }
}
