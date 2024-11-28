use conductor::prelude::*;
use std::sync::{Arc, RwLock};

use crate::settings::{FftSize, SampleRate};

struct ChartRunner {
    data: Arc<RwLock<Vec<[f64; 2]>>>,

    input: NodeRunnerInputPort<Vec<f64>>,

    fft_size: NodeRunnerInputPort<FftSize>,
    sample_rate: NodeRunnerInputPort<SampleRate>,
}

impl NodeRunner for ChartRunner {
    fn run(self: Box<Self>) {
        fn index_to_hz(index: usize, fft_size: FftSize, sample_rate: SampleRate) -> f64 {
            index as f64 * (sample_rate as f64 / fft_size as f64)
        }

        let mut fft_size = self.fft_size.recv().unwrap();
        let mut sample_rate = self.sample_rate.recv().unwrap();

        loop {
            receive! {
                (self.input): buffer => {
                    *self.data.write().unwrap() = buffer
                        .into_iter()
                        .enumerate()
                        .map(|(i, v)| [index_to_hz(i, fft_size, sample_rate), v])
                        .collect();
                },
                (self.fft_size): new_fft_size => {
                    fft_size = new_fft_size;
                },
                (self.sample_rate): new_sample_rate => {
                    sample_rate = new_sample_rate;
                },
            };
        }
    }
}

pub struct Chart {
    data: Arc<RwLock<Vec<[f64; 2]>>>,

    pub input: NodeConfigInputPort<Vec<f64>>,

    pub fft_size: NodeConfigInputPort<FftSize>,
    pub sample_rate: NodeConfigInputPort<SampleRate>,
}

impl Chart {
    pub fn new(data: Arc<RwLock<Vec<[f64; 2]>>>) -> Self {
        Self {
            data,

            input: NodeConfigInputPort::new(),

            fft_size: NodeConfigInputPort::new(),
            sample_rate: NodeConfigInputPort::new(),
        }
    }
}

impl NodeConfig for Chart {
    fn into_runner(self: Box<Self>) -> Box<dyn NodeRunner + Send> {
        Box::new(ChartRunner {
            data: self.data,

            input: self.input.into(),

            fft_size: self.fft_size.into(),
            sample_rate: self.sample_rate.into(),
        })
    }
}
