mod chart;
mod trigger;

use crate::{
    application::{Application, CHART_X_BOUND_MARGIN},
    settings::Periods,
    PeakVoltmeterPacket,
};
use chart::Chart;
use conductor::{core::pipeline::Pipeline, prelude::*};
use egui::{Color32, RichText, Vec2b};
use egui_plot::{Line, Plot, PlotPoints};
use std::sync::{Arc, RwLock};
use trigger::RisingEdgeTrigger;

pub struct TimeChartInputPorts {
    pub data: NodeConfigInputPort<PeakVoltmeterPacket>,
    pub periods: NodeConfigInputPort<Periods>,
}

pub fn time_chart(data: Arc<RwLock<Vec<f64>>>) -> Pipeline<TimeChartInputPorts, ()> {
    let into_i32 = Intoer::<_, i32>::new();

    let trigger = RisingEdgeTrigger::new(0);

    let period = Downsampler::new();

    let buffer = Chart::new(data);

    into_i32.output.connect(&trigger.input);
    into_i32.output.connect(&buffer.input);

    trigger.trigger.connect(&period.input);

    period.output.connect(&buffer.trigger);

    let input_ports = TimeChartInputPorts {
        data: into_i32.input.clone(),
        periods: period.factor.clone(),
    };

    Pipeline::new(
        vec![
            Box::new(into_i32),
            Box::new(trigger),
            Box::new(period),
            Box::new(buffer),
        ],
        input_ports,
        (),
    )
}

pub struct TimeChart {
    pub data: Arc<RwLock<Vec<f64>>>,
}

impl TimeChart {
    pub fn new(data: Arc<RwLock<Vec<f64>>>) -> Self {
        Self { data }
    }

    pub fn ui(&self, ui: &mut egui::Ui, application: &Application) {
        let available_size = ui.available_size();

        ui.allocate_ui_with_layout(
            egui::vec2(available_size.x, available_size.y / 2.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.y = 10.0;

                ui.label(RichText::new("Time Chart").size(20.0).strong());

                let x_bound = Self::sample_to_time(
                    application.chart_x_bound + CHART_X_BOUND_MARGIN,
                    application,
                );

                let mut plot = Plot::new("Plot")
                    .auto_bounds(Vec2b::new(false, true))
                    .y_axis_label("Voltage")
                    .x_axis_label("Time")
                    .allow_boxed_zoom(false)
                    .allow_drag(false)
                    .allow_zoom(false)
                    .allow_scroll(false)
                    .include_x(0.0)
                    .include_x(x_bound);

                // We need to check if the x bound has changed to reset the plot, otherwise the
                // plot will not update the x bound.
                let bound_changed = ui.memory_mut(|mem| {
                    let prev_x_bound = mem
                        .data
                        .get_temp::<f64>("prev_x_bound".into())
                        .unwrap_or(f64::NEG_INFINITY);

                    mem.data.insert_temp("prev_x_bound".into(), x_bound);

                    (prev_x_bound - x_bound).abs() > f64::EPSILON
                });

                if bound_changed {
                    plot = plot.reset()
                }

                plot.show(ui, |plot_ui| {
                    plot_ui.line(self.signal(application));
                });
            },
        );
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
