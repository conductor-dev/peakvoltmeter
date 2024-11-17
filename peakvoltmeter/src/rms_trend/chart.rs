use crate::{RMS_WINDOW, SAMPLE_RATE};
use conductor::prelude::*;
use std::sync::{Arc, RwLock};

struct ChartRunner {
    data: Arc<RwLock<Vec<f64>>>,

    input: NodeRunnerInputPort<i32>,
}

impl NodeRunner for ChartRunner {
    fn run(self: Box<Self>) {
        let mut rms_data = Vec::new();

        let mut buffer = CircularBuffer::new((RMS_WINDOW * SAMPLE_RATE as f32) as usize);

        loop {
            buffer.push(self.input.recv().unwrap());

            let rms = (buffer
                .iter()
                .fold(0.0, |acc, &v| acc + (v as f64 * v as f64))
                / buffer.len() as f64)
                .sqrt();

            rms_data.push(rms);

            *self.data.write().unwrap() = rms_data.clone();
        }
    }
}

pub struct Chart {
    data: Arc<RwLock<Vec<f64>>>,

    pub input: NodeConfigInputPort<i32>,
}

impl Chart {
    pub fn new(data: Arc<RwLock<Vec<f64>>>) -> Self {
        Self {
            data,

            input: NodeConfigInputPort::new(),
        }
    }
}

impl NodeConfig for Chart {
    fn into_runner(self: Box<Self>) -> Box<dyn NodeRunner + Send> {
        Box::new(ChartRunner {
            data: self.data,

            input: self.input.into(),
        })
    }
}
