use crate::{RMS_WINDOW, SAMPLE_RATE};
use conductor::{
    core::{NodeConfig, NodeRunner},
    prelude::{CircularBuffer, NodeConfigInputPort, NodeRunnerInputPort},
};
use egui::{Color32, RichText};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use std::sync::{Arc, RwLock};

struct RmsTrendRunner {
    data: Arc<RwLock<Vec<f64>>>,

    input: NodeRunnerInputPort<i32>,
}

impl NodeRunner for RmsTrendRunner {
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

pub struct RmsTrend {
    data: Arc<RwLock<Vec<f64>>>,

    pub input: NodeConfigInputPort<i32>,
}

impl RmsTrend {
    pub fn new(data: Arc<RwLock<Vec<f64>>>) -> Self {
        Self {
            data,

            input: NodeConfigInputPort::new(),
        }
    }
}

impl NodeConfig for RmsTrend {
    fn into_runner(self: Box<Self>) -> Box<dyn NodeRunner + Send> {
        Box::new(RmsTrendRunner {
            data: self.data,

            input: self.input.into(),
        })
    }
}

pub struct RmsTrendUi {
    data: Arc<RwLock<Vec<f64>>>,
}

impl RmsTrendUi {
    pub fn new(data: Arc<RwLock<Vec<f64>>>) -> Self {
        Self { data }
    }

    pub fn ui(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 10.0;

            ui.label(RichText::new("RMS Trend").size(20.0).strong());

            let plot = Plot::new("RmsTrend")
                .legend(Legend::default())
                .y_axis_label("Voltage")
                .x_axis_label("Time")
                .allow_boxed_zoom(false)
                .allow_drag(false)
                .allow_zoom(false)
                .allow_scroll(false);

            plot.show(ui, |plot_ui| {
                plot_ui.line(self.signal());
            });
        });
    }

    fn sample_to_time(&self, sample: usize) -> f64 {
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
                .map(|(i, v)| [self.sample_to_time(i), v]),
        );

        Line::new(plot_points)
            .color(Color32::LIGHT_RED)
            .name("Signal")
    }
}
