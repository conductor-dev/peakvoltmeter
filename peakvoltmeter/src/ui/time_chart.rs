use crate::trigger::TriggerMessage;
use conductor::{
    core::{receive, NodeConfig, NodeRunner},
    prelude::{NodeConfigInputPort, NodeRunnerInputPort},
};
use egui::{Color32, RichText, Vec2b};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use std::sync::{Arc, RwLock};

struct TimeChartRunner {
    data: Arc<RwLock<Vec<i32>>>,

    trigger: NodeRunnerInputPort<TriggerMessage>,
    input: NodeRunnerInputPort<i32>,
}

impl NodeRunner for TimeChartRunner {
    fn run(self: Box<Self>) {
        let mut cache = Vec::new();

        loop {
            receive! {
                (self.trigger): _msg => {
                    *self.data.write().unwrap() = std::mem::take(&mut cache);
                },
                (self.input): msg => {
                    cache.push(msg);
                },
            };
        }
    }
}

pub struct TimeChart {
    data: Arc<RwLock<Vec<i32>>>,

    pub trigger: NodeConfigInputPort<TriggerMessage>,
    pub input: NodeConfigInputPort<i32>,
}

impl TimeChart {
    pub fn new(data: Arc<RwLock<Vec<i32>>>) -> Self {
        Self {
            data,

            trigger: NodeConfigInputPort::new(),
            input: NodeConfigInputPort::new(),
        }
    }
}

impl NodeConfig for TimeChart {
    fn into_runner(self: Box<Self>) -> Box<dyn NodeRunner + Send> {
        Box::new(TimeChartRunner {
            data: self.data,

            trigger: self.trigger.into(),
            input: self.input.into(),
        })
    }
}

pub struct TimeChartUi {
    data: Arc<RwLock<Vec<i32>>>,
}

impl TimeChartUi {
    pub fn new(data: Arc<RwLock<Vec<i32>>>) -> Self {
        Self { data }
    }

    pub fn ui(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 10.0;

            ui.label(RichText::new("Time Chart").size(20.0).strong());

            let plot = Plot::new("Plot")
                .auto_bounds(Vec2b::FALSE)
                .legend(Legend::default())
                .y_axis_label("Voltage")
                .x_axis_label("Time")
                .include_y(-1_500_000)
                .include_y(1_500_000)
                .include_x(0.0)
                .include_x(200);

            plot.show(ui, |plot_ui| {
                plot_ui.line(self.signal());
            });
        });
    }

    fn signal(&self) -> Line {
        let plot_points = PlotPoints::from_iter(
            self.data
                .read()
                .unwrap()
                .clone()
                .into_iter()
                .enumerate()
                .map(|(i, v)| [i as f64, v as f64]),
        );

        Line::new(plot_points)
            .color(Color32::LIGHT_RED)
            .name("Signal")
    }
}
