mod chart;
mod trigger;

use crate::{
    application::CHART_X_BOUND_MARGIN,
    settings::{SampleRate, TimeChartPeriods},
};
use chart::Chart;
use conductor::{core::pipeline::Pipeline, prelude::*};
use egui::{Color32, RichText, Vec2b};
use egui_plot::{Line, Plot, PlotPoints};
use std::sync::{Arc, RwLock};
use trigger::RisingEdgeTrigger;

pub struct TimeChartInputPorts {
    pub data: (NodeConfigInputPort<f32>, NodeConfigInputPort<f32>),
    pub periods: NodeConfigInputPort<TimeChartPeriods>,
    pub sample_rate: NodeConfigInputPort<SampleRate>,
}

pub fn time_chart(data: Arc<RwLock<Vec<[f64; 2]>>>) -> Pipeline<TimeChartInputPorts, ()> {
    let trigger = RisingEdgeTrigger::new(0.0);

    let period = Downsampler::new();

    let chart = Chart::new(data);

    trigger.trigger.connect(&period.input);

    period.output.connect(&chart.trigger);

    let input_ports = TimeChartInputPorts {
        data: (trigger.input.clone(), chart.input.clone()),
        periods: period.factor.clone(),
        sample_rate: chart.sample_rate.clone(),
    };

    Pipeline::new(
        vec![Box::new(trigger), Box::new(period), Box::new(chart)],
        input_ports,
        (),
    )
}

pub struct TimeChart {
    pub data: Arc<RwLock<Vec<[f64; 2]>>>,

    prev_x_bound: f64,
}

impl TimeChart {
    pub fn new(data: Arc<RwLock<Vec<[f64; 2]>>>) -> Self {
        Self {
            data,
            prev_x_bound: f64::NEG_INFINITY,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, chart_x_bound: usize, sample_rate: usize) {
        let available_size = ui.available_size();

        ui.allocate_ui_with_layout(
            egui::vec2(available_size.x, available_size.y / 2.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.y = 10.0;

                ui.label(RichText::new("Time Chart").size(20.0).strong());

                let x_bound = (chart_x_bound + CHART_X_BOUND_MARGIN) as f64 / sample_rate as f64;

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
                if (self.prev_x_bound - x_bound).abs() > f64::EPSILON {
                    plot = plot.reset();
                    self.prev_x_bound = x_bound;
                }

                plot.show(ui, |plot_ui| {
                    plot_ui.line(self.signal());
                });
            },
        );
    }

    fn signal(&self) -> Line {
        let plot_points = PlotPoints::from_iter(self.data.read().unwrap().clone());

        Line::new(plot_points)
            .color(Color32::LIGHT_BLUE)
            .name("Signal")
    }
}
