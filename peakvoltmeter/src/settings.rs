use conductor::prelude::*;
use std::sync::mpsc::Receiver;

pub type SampleRate = usize;
pub type TimeChartPeriods = usize;
pub type FftSize = usize;
pub type RmsWindow = f32;
pub type RmsChartSize = usize;
pub type RmsRefreshPeriod = f32;

pub enum SettingsPacket {
    // general settings
    SampleRate(SampleRate),

    // time chart settings
    TimeChartPeriods(TimeChartPeriods),

    // harmonics settings
    FftSize(FftSize),

    // rms trend settings
    RmsWindow(RmsWindow),
    RmsChartSize(RmsChartSize),
    RmsRefreshPeriod(RmsRefreshPeriod),
}

struct SettingsRunner {
    receiver: Receiver<SettingsPacket>,

    sample_rate: NodeRunnerOutputPort<SampleRate>,
    time_chartperiods: NodeRunnerOutputPort<TimeChartPeriods>,
    fft_size: NodeRunnerOutputPort<FftSize>,
    rms_window: NodeRunnerOutputPort<RmsWindow>,
    rms_chart_size: NodeRunnerOutputPort<RmsChartSize>,
    rms_refresh_period: NodeRunnerOutputPort<RmsRefreshPeriod>,
}

impl NodeRunner for SettingsRunner {
    fn run(self: Box<Self>) {
        loop {
            let value = self.receiver.recv().unwrap();

            match value {
                SettingsPacket::SampleRate(sample_rate) => {
                    self.sample_rate.send(&sample_rate);
                }
                SettingsPacket::TimeChartPeriods(periods) => {
                    self.time_chartperiods.send(&periods);
                }
                SettingsPacket::FftSize(fft_size) => {
                    self.fft_size.send(&fft_size);
                }
                SettingsPacket::RmsWindow(rms_window) => {
                    self.rms_window.send(&rms_window);
                }
                SettingsPacket::RmsChartSize(rms_chart_size) => {
                    self.rms_chart_size.send(&rms_chart_size);
                }
                SettingsPacket::RmsRefreshPeriod(rms_refresh_period) => {
                    self.rms_refresh_period.send(&rms_refresh_period);
                }
            }
        }
    }
}

pub struct Settings {
    receiver: Receiver<SettingsPacket>,

    pub sample_rate: NodeConfigOutputPort<SampleRate>,
    pub time_chart_periods: NodeConfigOutputPort<TimeChartPeriods>,
    pub fft_size: NodeConfigOutputPort<FftSize>,
    pub rms_window: NodeConfigOutputPort<RmsWindow>,
    pub rms_chart_size: NodeConfigOutputPort<RmsChartSize>,
    pub rms_refresh_period: NodeConfigOutputPort<RmsRefreshPeriod>,
}

impl Settings {
    pub fn new(receiver: Receiver<SettingsPacket>) -> Self {
        Self {
            receiver,

            sample_rate: NodeConfigOutputPort::new(),
            time_chart_periods: NodeConfigOutputPort::new(),
            fft_size: NodeConfigOutputPort::new(),
            rms_window: NodeConfigOutputPort::new(),
            rms_chart_size: NodeConfigOutputPort::new(),
            rms_refresh_period: NodeConfigOutputPort::new(),
        }
    }
}

impl NodeConfig for Settings {
    fn into_runner(self: Box<Self>) -> Box<dyn NodeRunner + Send> {
        Box::new(SettingsRunner {
            receiver: self.receiver,
            sample_rate: self.sample_rate.into(),
            time_chartperiods: self.time_chart_periods.into(),
            fft_size: self.fft_size.into(),
            rms_window: self.rms_window.into(),
            rms_chart_size: self.rms_chart_size.into(),
            rms_refresh_period: self.rms_refresh_period.into(),
        })
    }
}
