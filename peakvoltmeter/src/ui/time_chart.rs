use crate::{trigger::TriggerMessage, SAMPLE_RATE};
use conductor::{
    core::{receive, NodeConfig, NodeRunner},
    prelude::{NodeConfigInputPort, NodeRunnerInputPort},
};
use egui::{Color32, RichText, Vec2b};
use egui_plot::{Line, Plot, PlotPoints};
use std::sync::{Arc, RwLock};

struct TimeChartRunner {
    data: Arc<RwLock<Vec<f64>>>,

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
                    cache.push(msg.into());
                },
            };
        }
    }
}

pub struct TimeChart {
    data: Arc<RwLock<Vec<f64>>>,

    pub trigger: NodeConfigInputPort<TriggerMessage>,
    pub input: NodeConfigInputPort<i32>,
}

impl TimeChart {
    pub fn new(data: Arc<RwLock<Vec<f64>>>) -> Self {
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
    data: Arc<RwLock<Vec<f64>>>,
}

impl TimeChartUi {
    pub fn new(data: Arc<RwLock<Vec<f64>>>) -> Self {
        Self { data }
    }

    pub fn ui(&self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();

        ui.allocate_ui_with_layout(
            egui::vec2(available_size.x, available_size.y / 2.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.y = 10.0;

                ui.label(RichText::new("Time Chart").size(20.0).strong());

                let plot = Plot::new("Plot")
                    .auto_bounds(Vec2b::new(false, true))
                    .y_axis_label("Voltage")
                    .x_axis_label("Time")
                    .allow_boxed_zoom(false)
                    .allow_drag(false)
                    .allow_zoom(false)
                    .allow_scroll(false)
                    .include_x(0.0)
                    .include_x(Self::sample_to_time(200));

                plot.show(ui, |plot_ui| {
                    plot_ui.line(self.signal());
                });
            },
        );
    }

    fn sample_to_time(sample: usize) -> f64 {
        sample as f64 * (1.0 / SAMPLE_RATE as f64)
    }

    fn signal(&self) -> Line {
        let plot_points = PlotPoints::from_iter(
            self.data
                .read()
                .unwrap()
                .clone()
                .into_iter()
                .enumerate()
                .map(|(i, v)| [Self::sample_to_time(i), v]),
        );

        Line::new(plot_points)
            .color(Color32::LIGHT_BLUE)
            .name("Signal")
    }
}
