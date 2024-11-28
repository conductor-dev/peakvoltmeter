use conductor::prelude::*;
use std::sync::mpsc::Receiver;

pub type SampleRate = f32;
pub type AdcCalibrationFactor = f32;
pub type HvDividerFactor = f32;
pub type TimeChartPeriods = usize;
pub type FftSize = usize;
pub type RmsWindow = f32;
pub type ChartSize = usize;
pub type RefreshPeriod = f32;

pub enum SettingsPacket {
    // signal settings
    SampleRate(SampleRate),
    AdcCalibrationFactor(AdcCalibrationFactor),
    HvDividerFactor(HvDividerFactor),

    // time chart settings
    TimeChartPeriods(TimeChartPeriods),

    // harmonics settings
    FftSize(FftSize),
    HarmonicsRefreshPeriod(RefreshPeriod),

    // rms trend and peak sqrt settings
    Window(RmsWindow),
    ChartSize(ChartSize),
    RmsRefreshPeriod(RefreshPeriod),
}

struct SettingsRunner {
    receiver: Receiver<SettingsPacket>,

    sample_rate: NodeRunnerOutputPort<SampleRate>,
    adc_calibration_factor: NodeRunnerOutputPort<AdcCalibrationFactor>,
    hv_divider_factor: NodeRunnerOutputPort<HvDividerFactor>,
    time_chartperiods: NodeRunnerOutputPort<TimeChartPeriods>,
    fft_size: NodeRunnerOutputPort<FftSize>,
    harmonics_refresh_period: NodeRunnerOutputPort<RefreshPeriod>,
    window: NodeRunnerOutputPort<RmsWindow>,
    chart_size: NodeRunnerOutputPort<ChartSize>,
    rms_refresh_period: NodeRunnerOutputPort<RefreshPeriod>,
}

impl NodeRunner for SettingsRunner {
    fn run(self: Box<Self>) {
        loop {
            let Ok(value) = self.receiver.recv() else {
                break;
            };

            match value {
                SettingsPacket::SampleRate(sample_rate) => {
                    self.sample_rate.send(&sample_rate);
                }
                SettingsPacket::AdcCalibrationFactor(adc_calibration_factor) => {
                    self.adc_calibration_factor.send(&adc_calibration_factor);
                }
                SettingsPacket::HvDividerFactor(hv_divider_factor) => {
                    self.hv_divider_factor.send(&hv_divider_factor);
                }
                SettingsPacket::TimeChartPeriods(periods) => {
                    self.time_chartperiods.send(&periods);
                }
                SettingsPacket::FftSize(fft_size) => {
                    self.fft_size.send(&fft_size);
                }
                SettingsPacket::HarmonicsRefreshPeriod(refresh_period) => {
                    self.harmonics_refresh_period.send(&refresh_period);
                }
                SettingsPacket::Window(window) => {
                    self.window.send(&window);
                }
                SettingsPacket::ChartSize(chart_size) => {
                    self.chart_size.send(&chart_size);
                }
                SettingsPacket::RmsRefreshPeriod(refresh_period) => {
                    self.rms_refresh_period.send(&refresh_period);
                }
            }
        }
    }
}

pub struct Settings {
    receiver: Receiver<SettingsPacket>,

    pub sample_rate: NodeConfigOutputPort<SampleRate>,
    pub adc_calibration_factor: NodeConfigOutputPort<AdcCalibrationFactor>,
    pub hv_divider_factor: NodeConfigOutputPort<HvDividerFactor>,
    pub time_chart_periods: NodeConfigOutputPort<TimeChartPeriods>,
    pub fft_size: NodeConfigOutputPort<FftSize>,
    pub harmonics_refresh_period: NodeConfigOutputPort<RefreshPeriod>,
    pub window: NodeConfigOutputPort<RmsWindow>,
    pub chart_size: NodeConfigOutputPort<ChartSize>,
    pub rms_refresh_period: NodeConfigOutputPort<RefreshPeriod>,
}

impl Settings {
    pub fn new(receiver: Receiver<SettingsPacket>) -> Self {
        Self {
            receiver,

            sample_rate: NodeConfigOutputPort::new(),
            adc_calibration_factor: NodeConfigOutputPort::new(),
            hv_divider_factor: NodeConfigOutputPort::new(),
            time_chart_periods: NodeConfigOutputPort::new(),
            fft_size: NodeConfigOutputPort::new(),
            harmonics_refresh_period: NodeConfigOutputPort::new(),
            window: NodeConfigOutputPort::new(),
            chart_size: NodeConfigOutputPort::new(),
            rms_refresh_period: NodeConfigOutputPort::new(),
        }
    }
}

impl NodeConfig for Settings {
    fn into_runner(self: Box<Self>) -> Box<dyn NodeRunner + Send> {
        Box::new(SettingsRunner {
            receiver: self.receiver,
            sample_rate: self.sample_rate.into(),
            adc_calibration_factor: self.adc_calibration_factor.into(),
            hv_divider_factor: self.hv_divider_factor.into(),
            time_chartperiods: self.time_chart_periods.into(),
            fft_size: self.fft_size.into(),
            harmonics_refresh_period: self.harmonics_refresh_period.into(),
            window: self.window.into(),
            chart_size: self.chart_size.into(),
            rms_refresh_period: self.rms_refresh_period.into(),
        })
    }
}
