use crate::settings::{RmsChartSize, RmsRefreshPeriod};
use conductor::prelude::*;
use std::sync::{Arc, RwLock};

struct ChartRunner {
    data: Arc<RwLock<Vec<f64>>>,

    input: NodeRunnerInputPort<Vec<i32>>,

    chart_size: NodeRunnerInputPort<RmsChartSize>,
    refresh_period: NodeRunnerInputPort<RmsRefreshPeriod>,
}

impl NodeRunner for ChartRunner {
    fn run(self: Box<Self>) {
        let mut chart_size = self.chart_size.recv().unwrap();
        let mut refresh_period = self.refresh_period.recv().unwrap();

        let mut rms_data = CircularBuffer::new((chart_size as f32 / refresh_period) as usize);

        loop {
            receive! {
                (self.input): buffer => {
                    let rms = (buffer
                        .iter()
                        .fold(0.0, |acc, &v| acc + (v as f64 * v as f64))
                        / buffer.len() as f64)
                        .sqrt();

                    rms_data.push(rms);

                    *self.data.write().unwrap() = rms_data.clone().into();
                },
                (self.chart_size): new_chart_size => {
                    chart_size = new_chart_size;

                    rms_data.resize((chart_size as f32 / refresh_period) as usize);
                },
                (self.refresh_period): new_refresh_period => {
                    refresh_period = new_refresh_period;

                    // previous data is invalidated so new buffer must be created
                    rms_data = CircularBuffer::new((chart_size as f32 / refresh_period) as usize);
                },
            };
        }
    }
}

pub struct Chart {
    data: Arc<RwLock<Vec<f64>>>,

    pub input: NodeConfigInputPort<Vec<i32>>,

    pub chart_size: NodeConfigInputPort<RmsChartSize>,
    pub refresh_period: NodeConfigInputPort<RmsRefreshPeriod>,
}

impl Chart {
    pub fn new(data: Arc<RwLock<Vec<f64>>>) -> Self {
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
