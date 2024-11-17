use super::trigger::TriggerMessage;
use conductor::prelude::*;
use std::sync::{Arc, RwLock};

struct ChartRunner {
    data: Arc<RwLock<Vec<f64>>>,

    trigger: NodeRunnerInputPort<TriggerMessage>,
    input: NodeRunnerInputPort<i32>,
}

impl NodeRunner for ChartRunner {
    fn run(self: Box<Self>) {
        let mut cache = Vec::new();

        loop {
            receive! {
                (self.trigger): _msg => {
                    *self.data.write().unwrap() = std::mem::take(&mut cache);
                },
                (self.input): msg => {
                    cache.push(msg.into());
                },
            };
        }
    }
}

pub struct Chart {
    data: Arc<RwLock<Vec<f64>>>,

    pub trigger: NodeConfigInputPort<TriggerMessage>,
    pub input: NodeConfigInputPort<i32>,
}

impl Chart {
    pub fn new(data: Arc<RwLock<Vec<f64>>>) -> Self {
        Self {
            data,

            trigger: NodeConfigInputPort::new(),
            input: NodeConfigInputPort::new(),
        }
    }
}

impl NodeConfig for Chart {
    fn into_runner(self: Box<Self>) -> Box<dyn NodeRunner + Send> {
        Box::new(ChartRunner {
            data: self.data,

            trigger: self.trigger.into(),
            input: self.input.into(),
        })
    }
}
