use conductor::prelude::*;
use std::sync::mpsc::Receiver;

pub type SampleRate = usize;
pub type FftSize = usize;
pub type RmsWindow = f32;

pub enum SettingsPacket {
    SampleRate(SampleRate),
    FftSize(FftSize),
    RmsWindow(RmsWindow),
}

struct SettingsRunner {
    receiver: Receiver<SettingsPacket>,

    sample_rate: NodeRunnerOutputPort<SampleRate>,
    fft_size: NodeRunnerOutputPort<FftSize>,
    rms_window: NodeRunnerOutputPort<RmsWindow>,
}

impl NodeRunner for SettingsRunner {
    fn run(self: Box<Self>) {
        loop {
            let value = self.receiver.recv().unwrap();

            match value {
                SettingsPacket::SampleRate(sample_rate) => {
                    self.sample_rate.send(&sample_rate);
                }
                SettingsPacket::FftSize(fft_size) => {
                    self.fft_size.send(&fft_size);
                }
                SettingsPacket::RmsWindow(rms_window) => {
                    self.rms_window.send(&rms_window);
                }
            }
        }
    }
}

pub struct Settings {
    receiver: Receiver<SettingsPacket>,

    pub sample_rate: NodeConfigOutputPort<SampleRate>,
    pub fft_size: NodeConfigOutputPort<FftSize>,
    pub rms_window: NodeConfigOutputPort<RmsWindow>,
}

impl Settings {
    pub fn new(receiver: Receiver<SettingsPacket>) -> Self {
        Self {
            receiver,

            sample_rate: NodeConfigOutputPort::new(),
            fft_size: NodeConfigOutputPort::new(),
            rms_window: NodeConfigOutputPort::new(),
        }
    }
}

impl NodeConfig for Settings {
    fn into_runner(self: Box<Self>) -> Box<dyn NodeRunner + Send> {
        Box::new(SettingsRunner {
            receiver: self.receiver,
            sample_rate: self.sample_rate.into(),
            fft_size: self.fft_size.into(),
            rms_window: self.rms_window.into(),
        })
    }
}
