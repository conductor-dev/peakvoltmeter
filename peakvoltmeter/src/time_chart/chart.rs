use crate::settings::SampleRate;

use super::trigger::TriggerMessage;
use conductor::prelude::*;
use std::sync::{Arc, RwLock};

struct ChartRunner {
    data: Arc<RwLock<Vec<[f64; 2]>>>,

    trigger: NodeRunnerInputPort<TriggerMessage>,
    input: NodeRunnerInputPort<i32>,

    sample_rate: NodeRunnerInputPort<SampleRate>,
}

impl NodeRunner for ChartRunner {
    fn run(self: Box<Self>) {
        fn index_to_time(index: usize, sample_rate: usize) -> f64 {
            index as f64 * (1.0 / sample_rate as f64)
        }

        let mut cache = Vec::new();

        let mut sample_rate = self.sample_rate.recv().unwrap();

        loop {
            receive! {
                (self.trigger): _msg => {
                    *self.data.write().unwrap() = std::mem::take(&mut cache)
                        .into_iter()
                        .enumerate()
                        .map(|(i, v)| [index_to_time(i, sample_rate), v as f64])
                        .collect();
                },
                (self.input): msg => {
                    cache.push(msg);
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

    pub trigger: NodeConfigInputPort<TriggerMessage>,
    pub input: NodeConfigInputPort<i32>,

    pub sample_rate: NodeConfigInputPort<SampleRate>,
}

impl Chart {
    pub fn new(data: Arc<RwLock<Vec<[f64; 2]>>>) -> Self {
        Self {
            data,

            trigger: NodeConfigInputPort::new(),
            input: NodeConfigInputPort::new(),

            sample_rate: NodeConfigInputPort::new(),
        }
    }
}

impl NodeConfig for Chart {
    fn into_runner(self: Box<Self>) -> Box<dyn NodeRunner + Send> {
        Box::new(ChartRunner {
            data: self.data,

            trigger: self.trigger.into(),
            input: self.input.into(),

            sample_rate: self.sample_rate.into(),
        })
    }
}
