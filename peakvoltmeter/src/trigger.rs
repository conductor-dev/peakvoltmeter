use conductor::{
    core::{NodeConfig, NodeRunner},
    prelude::{
        NodeConfigInputPort, NodeConfigOutputPort, NodeRunnerInputPort, NodeRunnerOutputPort,
    },
};

#[derive(Clone)]
pub enum TriggerMessage {
    Triggered,
}

struct RisingEdgeTriggerRunner<T> {
    threshold: T,

    input: NodeRunnerInputPort<T>,
    trigger: NodeRunnerOutputPort<TriggerMessage>,
}

impl<T: PartialOrd> NodeRunner for RisingEdgeTriggerRunner<T> {
    fn run(self: Box<Self>) {
        let mut previous_value = self.input.recv().unwrap();

        loop {
            let value = self.input.recv().unwrap();

            if previous_value < self.threshold && value >= self.threshold {
                self.trigger.send(&TriggerMessage::Triggered);
            }

            previous_value = value;
        }
    }
}

pub struct RisingEdgeTrigger<T: PartialOrd> {
    threshold: T,

    pub input: NodeConfigInputPort<T>,
    pub trigger: NodeConfigOutputPort<TriggerMessage>,
}

impl<T: PartialOrd> RisingEdgeTrigger<T> {
    pub fn new(threshold: T) -> Self {
        Self {
            threshold,

            input: NodeConfigInputPort::new(),
            trigger: NodeConfigOutputPort::new(),
        }
    }
}

impl<T: PartialOrd + Clone + Send + 'static> NodeConfig for RisingEdgeTrigger<T> {
    fn into_runner(self: Box<Self>) -> Box<dyn NodeRunner + Send> {
        Box::new(RisingEdgeTriggerRunner {
            threshold: self.threshold,

            input: self.input.into(),
            trigger: self.trigger.into(),
        })
    }
}
