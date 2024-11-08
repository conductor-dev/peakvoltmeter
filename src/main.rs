mod trigger;
mod ui;

use conductor::prelude::*;
use std::{
    sync::{Arc, RwLock},
    thread,
};
use ui::{Ui, UiApp};

fn main() {
    let buffer = Arc::new(RwLock::new(CircularBuffer::new(100_000)));

    let udp_receiver = UdpReceiver::<PeakVoltmeterPacket>::new("127.0.0.1:8080");
    let into = Intoer::new();
    let ui = Ui::new(buffer.clone());

    udp_receiver.output.connect(&into.input);
    into.output.connect(&ui.input);

    thread::spawn(move || {
        pipeline!(udp_receiver, into, ui).run();
    });

    eframe::run_native(
        "Plotter",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(UiApp::new(buffer)))),
    )
    .unwrap();
}

#[derive(Clone, Copy)]
struct PeakVoltmeterPacket(f32);

impl From<PeakVoltmeterPacket> for f32 {
    fn from(packet: PeakVoltmeterPacket) -> Self {
        packet.0
    }
}

impl UdpDeserializer for PeakVoltmeterPacket {
    fn max_packet_size() -> usize {
        size_of::<f32>()
    }

    fn deserialize_packet(bytes: &[u8]) -> Self {
        // Self(i32::from_be_bytes([bytes[5], bytes[6], bytes[7], 0]))
        Self(f32::from_ne_bytes(bytes.try_into().unwrap()))
    }
}
