use crate::settings::{ChartSize, FftSize, RefreshPeriod, SampleRate};

use conductor::prelude::*;
use std::sync::{Arc, RwLock};

struct ChartRunner {
    data: Arc<RwLock<Vec<[f64; 2]>>>,

    fft_input: NodeRunnerInputPort<Vec<f64>>,

    chart_size: NodeRunnerInputPort<ChartSize>,
    sample_rate: NodeRunnerInputPort<SampleRate>,
    fft_size: NodeRunnerInputPort<FftSize>,
    refresh_period: NodeRunnerInputPort<RefreshPeriod>,
}

impl NodeRunner for ChartRunner {
    fn run(self: Box<Self>) {
        fn index_to_time(index: usize, buffer_size: usize, refresh_period: RefreshPeriod) -> f64 {
            (index as f64 - (buffer_size as f64 - 1.0)) * refresh_period as f64
        }

        fn calculate_buffer_size(chart_size: usize, refresh_period: RefreshPeriod) -> usize {
            (chart_size as f32 / refresh_period) as usize + 1
        }

        let mut chart_size = self.chart_size.recv();
        let mut sample_rate = self.sample_rate.recv();
        let mut fft_size = self.fft_size.recv();
        let mut refresh_period = self.refresh_period.recv();

        let mut frequency_data =
            CircularBuffer::new(calculate_buffer_size(chart_size, refresh_period));

        loop {
            receive! {
                (self.fft_input): buffer => {
                    let Some((max_index, _)) = buffer
                        .iter()
                        .enumerate()
                        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    else {
                        continue;
                    };

                    let frequency = max_index as f64 * (sample_rate as f64 / fft_size as f64);

                    frequency_data.push(frequency);

                    *self.data.write().unwrap() = frequency_data
                        .clone()
                        .into_iter()
                        .enumerate()
                        .map(|(i, v)| [index_to_time(i, frequency_data.len(), refresh_period), v])
                        .collect();
                },
                (self.chart_size): new_chart_size => {
                    chart_size = new_chart_size;

                    frequency_data.resize(calculate_buffer_size(chart_size, refresh_period));
                },
                (self.sample_rate): new_sample_rate => {
                    sample_rate = new_sample_rate;

                    // previous data is invalidated so new buffer must be created
                    frequency_data = CircularBuffer::new(calculate_buffer_size(chart_size, refresh_period));
                },
                (self.fft_size): new_fft_size => {
                    fft_size = new_fft_size;

                    // previous data is invalidated so new buffer must be created
                    frequency_data = CircularBuffer::new(calculate_buffer_size(chart_size, refresh_period));
                },
                (self.refresh_period): new_refresh_period => {
                    refresh_period = new_refresh_period;

                    // previous data is invalidated so new buffer must be created
                    frequency_data = CircularBuffer::new(calculate_buffer_size(chart_size, refresh_period));
                },
            };
        }
    }
}

pub struct Chart {
    data: Arc<RwLock<Vec<[f64; 2]>>>,

    pub fft_input: NodeConfigInputPort<Vec<f64>>,

    pub chart_size: NodeConfigInputPort<ChartSize>,
    pub sample_rate: NodeConfigInputPort<SampleRate>,
    pub fft_size: NodeConfigInputPort<FftSize>,
    pub refresh_period: NodeConfigInputPort<RefreshPeriod>,
}

impl Chart {
    pub fn new(data: Arc<RwLock<Vec<[f64; 2]>>>) -> Self {
        Self {
            data,

            fft_input: NodeConfigInputPort::new(),

            chart_size: NodeConfigInputPort::new(),
            sample_rate: NodeConfigInputPort::new(),
            fft_size: NodeConfigInputPort::new(),
            refresh_period: NodeConfigInputPort::new(),
        }
    }
}

impl NodeConfig for Chart {
    fn into_runner(self: Box<Self>) -> Box<dyn NodeRunner + Send> {
        Box::new(ChartRunner {
            data: self.data,

            fft_input: self.fft_input.into(),

            chart_size: self.chart_size.into(),
            sample_rate: self.sample_rate.into(),
            fft_size: self.fft_size.into(),
            refresh_period: self.refresh_period.into(),
        })
    }
}
