use crate::settings::{ChartSize, RefreshPeriod};
use conductor::prelude::*;
use std::sync::{Arc, RwLock};

struct ChartRunner {
    data: Arc<RwLock<Vec<[f64; 2]>>>,

    input: NodeRunnerInputPort<Vec<f32>>,

    chart_size: NodeRunnerInputPort<ChartSize>,
    refresh_period: NodeRunnerInputPort<RefreshPeriod>,
}

impl NodeRunner for ChartRunner {
    fn run(self: Box<Self>) {
        fn index_to_time(index: usize, buffer_size: usize, refresh_period: f32) -> f64 {
            (index as f64 - (buffer_size as f64 - 1.0)) * refresh_period as f64
        }

        fn calculate_buffer_size(chart_size: usize, refresh_period: f32) -> usize {
            (chart_size as f32 / refresh_period) as usize + 1
        }

        let mut chart_size = self.chart_size.recv();
        let mut refresh_period = self.refresh_period.recv();

        let mut rms_data = CircularBuffer::new(calculate_buffer_size(chart_size, refresh_period));

        loop {
            receive! {
                (self.input): buffer => {
                    let rms = (buffer
                        .iter()
                        .fold(0.0, |acc, &v| acc + (v as f64 * v as f64)) / buffer.len() as f64)
                        .sqrt();

                    rms_data.push(rms);

                    *self.data.write().unwrap() = rms_data
                        .clone()
                        .into_iter()
                        .enumerate()
                        .map(|(i, v)| [index_to_time(i, rms_data.len(), refresh_period), v])
                        .collect();
                },
                (self.chart_size): new_chart_size => {
                    chart_size = new_chart_size;

                    rms_data.resize(calculate_buffer_size(chart_size, refresh_period));
                },
                (self.refresh_period): new_refresh_period => {
                    refresh_period = new_refresh_period;

                    // previous data is invalidated so new buffer must be created
                    rms_data = CircularBuffer::new(calculate_buffer_size(chart_size, refresh_period));
                },
            };
        }
    }
}

pub struct Chart {
    data: Arc<RwLock<Vec<[f64; 2]>>>,

    pub input: NodeConfigInputPort<Vec<f32>>,

    pub chart_size: NodeConfigInputPort<ChartSize>,
    pub refresh_period: NodeConfigInputPort<RefreshPeriod>,
}

impl Chart {
    pub fn new(data: Arc<RwLock<Vec<[f64; 2]>>>) -> Self {
        Self {
            data,

            input: NodeConfigInputPort::new(),

            chart_size: NodeConfigInputPort::new(),
            refresh_period: NodeConfigInputPort::new(),
        }
    }
}

impl NodeConfig for Chart {
    fn into_runner(self: Box<Self>) -> Box<dyn NodeRunner + Send> {
        Box::new(ChartRunner {
            data: self.data,

            input: self.input.into(),

            chart_size: self.chart_size.into(),
            refresh_period: self.refresh_period.into(),
        })
    }
}
