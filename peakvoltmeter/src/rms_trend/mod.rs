mod chart;

use crate::{
    application::Application,
    settings::{RmsWindow, SampleRate},
    PeakVoltmeterPacket,
};
use chart::Chart;
use conductor::{core::pipeline::Pipeline, prelude::*};
use egui::{Color32, RichText};
use egui_plot::{Line, Plot, PlotPoints};
use std::sync::{Arc, RwLock};

pub struct RmsTrendInputPorts {
    pub data: NodeConfigInputPort<PeakVoltmeterPacket>,
    pub sample_rate: NodeConfigInputPort<SampleRate>,
    pub rms_window: NodeConfigInputPort<RmsWindow>,
}

pub fn rms_trend(data: Arc<RwLock<Vec<f64>>>) -> Pipeline<RmsTrendInputPorts, ()> {
    let into_i32 = Intoer::<_, i32>::new();

    let sample_rate_to_f32 = Lambdaer::new(|value: usize| value as f32);

    let buffer_size = Multiplier::new();

    let buffer_size_to_usize = Lambdaer::new(|value: f32| value as usize);

    let buffer = Buffer::new(true);

    let chart = Chart::new(data);

    into_i32.output.connect(&buffer.input);

    sample_rate_to_f32.output.connect(&buffer_size.input2);

    buffer_size.output.connect(&buffer_size_to_usize.input);

    buffer_size_to_usize.output.connect(&buffer.size);

    buffer.output.connect(&chart.input);

    let input_ports = RmsTrendInputPorts {
        data: into_i32.input.clone(),
        sample_rate: sample_rate_to_f32.input.clone(),
        rms_window: buffer_size.input1.clone(),
    };

    Pipeline::new(
        vec![
            Box::new(into_i32),
            Box::new(sample_rate_to_f32),
            Box::new(buffer_size),
            Box::new(buffer_size_to_usize),
            Box::new(buffer),
            Box::new(chart),
        ],
        input_ports,
        (),
    )
}

pub struct RmsTrend {
    data: Arc<RwLock<Vec<f64>>>,
}

impl RmsTrend {
    pub fn new(data: Arc<RwLock<Vec<f64>>>) -> Self {
        Self { data }
    }

    pub fn ui(&self, ui: &mut egui::Ui, application: &Application) {
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 10.0;

            ui.label(RichText::new("RMS Trend").size(20.0).strong());

            let plot = Plot::new("RmsTrend")
                .y_axis_label("Voltage")
                .x_axis_label("Time")
                .allow_boxed_zoom(false)
                .allow_drag(false)
                .allow_zoom(false)
                .allow_scroll(false);

            plot.show(ui, |plot_ui| {
                plot_ui.line(self.signal(application));
            });
        });
    }

    fn sample_to_time(sample: usize, application: &Application) -> f64 {
        sample as f64 * (1.0 / application.sample_rate as f64)
    }

    fn signal(&self, application: &Application) -> Line {
        let plot_points = PlotPoints::from_iter(
            self.data
                .read()
                .unwrap()
                .clone()
                .into_iter()
                .enumerate()
                .map(|(i, v)| [Self::sample_to_time(i, application), v]),
        );

        Line::new(plot_points)
            .color(Color32::LIGHT_BLUE)
            .name("Signal")
    }
}
