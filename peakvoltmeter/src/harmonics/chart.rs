use conductor::prelude::*;
use std::sync::{Arc, RwLock};

struct ChartRunner {
    data: Arc<RwLock<Vec<f64>>>,

    input: NodeRunnerInputPort<Vec<f64>>,
}

impl NodeRunner for ChartRunner {
    fn run(self: Box<Self>) {
        loop {
            let input = self.input.recv().unwrap();

            *self.data.write().unwrap() = input;
        }
    }
}

pub struct Chart {
    data: Arc<RwLock<Vec<f64>>>,

    pub input: NodeConfigInputPort<Vec<f64>>,
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
