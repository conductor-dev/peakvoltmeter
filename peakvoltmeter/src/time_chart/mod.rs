mod chart;
mod trigger;

use crate::{PeakVoltmeterPacket, SAMPLE_RATE};
use chart::Chart;
use conductor::{core::pipeline::Pipeline, prelude::*};
use egui::{Color32, RichText, Vec2b};
use egui_plot::{Line, Plot, PlotPoints};
use std::sync::{Arc, RwLock};
use trigger::RisingEdgeTrigger;

pub fn time_chart(
    data: Arc<RwLock<Vec<f64>>>,
) -> Pipeline<NodeConfigInputPort<PeakVoltmeterPacket>, ()> {
    let into_i32 = Intoer::<_, i32>::new();

    let trigger = RisingEdgeTrigger::new(0);

    let period = Downsampler::new(3);

    let buffer = Chart::new(data);

    into_i32.output.connect(&trigger.input);
    into_i32.output.connect(&buffer.input);

    trigger.trigger.connect(&period.input);

    period.output.connect(&buffer.trigger);

    let input = into_i32.input.clone();

    Pipeline::new(
        vec![
            Box::new(into_i32),
            Box::new(trigger),
            Box::new(period),
            Box::new(buffer),
        ],
        input,
        (),
    )
}

pub struct TimeChart {
    data: Arc<RwLock<Vec<f64>>>,
}

impl TimeChart {
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
