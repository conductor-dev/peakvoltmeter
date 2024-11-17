mod chart;

use crate::{PeakVoltmeterPacket, RMS_WINDOW, SAMPLE_RATE};
use chart::Chart;
use conductor::{core::pipeline::Pipeline, prelude::*};
use egui::{Color32, RichText};
use egui_plot::{Line, Plot, PlotPoints};
use std::sync::{Arc, RwLock};

pub fn rms_trend(
    data: Arc<RwLock<Vec<f64>>>,
) -> Pipeline<NodeConfigInputPort<PeakVoltmeterPacket>, ()> {
    let into_i32 = Intoer::<_, i32>::new();

    let buffer = Buffer::new(true);
    buffer
        .size
        .set_initial((RMS_WINDOW * SAMPLE_RATE as f32) as usize);

    let chart = Chart::new(data);

    into_i32.output.connect(&buffer.input);

    buffer.output.connect(&chart.input);

    let input = into_i32.input.clone();

    Pipeline::new(
        vec![Box::new(into_i32), Box::new(buffer), Box::new(chart)],
        input,
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

    pub fn ui(&self, ui: &mut egui::Ui) {
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
                plot_ui.line(self.signal());
            });
        });
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
