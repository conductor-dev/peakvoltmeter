mod chart;

use crate::{
    application::{calculate_precision, Precision, VoltageUnit},
    coordinates_formatter,
    settings::{ChartSize, RefreshPeriod, RmsWindow, SampleRate},
};
use chart::Chart;
use conductor::{core::pipeline::Pipeline, prelude::*};
use egui::{Color32, RichText, Vec2b};
use egui_plot::{Line, Plot, PlotPoints};
use std::sync::{Arc, RwLock};

pub struct RmsTrendInputPorts {
    pub data: NodeConfigInputPort<f32>,
    pub sample_rate: (
        NodeConfigInputPort<SampleRate>,
        NodeConfigInputPort<SampleRate>,
    ),
    pub window: NodeConfigInputPort<RmsWindow>,
    pub chart_size: NodeConfigInputPort<ChartSize>,
    pub refresh_preiod: (
        NodeConfigInputPort<RefreshPeriod>,
        NodeConfigInputPort<RefreshPeriod>,
    ),
}

pub struct RmsTrendOutputPorts {
    pub windowed_downsampled_data: NodeConfigOutputPort<Vec<f32>>,
}

pub fn rms_trend(
    data: Arc<RwLock<Vec<[f64; 2]>>>,
) -> Pipeline<RmsTrendInputPorts, RmsTrendOutputPorts> {
    let buffer_size = Multiply::new();

    let buffer_size_to_usize = Lambda::new(|value: f32| value as usize);

    let buffer = Buffer::new(true);

    let refresh_factor = Multiply::new();

    let refresh_factor_to_usize = Lambda::new(|value: f32| value as usize);

    let refresh_period_downsampler = Downsample::new();

    let chart = Chart::new(data);

    buffer_size.output.connect(&buffer_size_to_usize.input);

    buffer_size_to_usize.output.connect(&buffer.size);

    refresh_factor
        .output
        .connect(&refresh_factor_to_usize.input);

    refresh_factor_to_usize
        .output
        .connect(&refresh_period_downsampler.factor);

    buffer.output.connect(&refresh_period_downsampler.input);

    refresh_period_downsampler.output.connect(&chart.input);

    let input_ports = RmsTrendInputPorts {
        data: buffer.input.clone(),
        sample_rate: (buffer_size.input2.clone(), refresh_factor.input2.clone()),
        window: buffer_size.input1.clone(),
        chart_size: chart.chart_size.clone(),
        refresh_preiod: (refresh_factor.input1.clone(), chart.refresh_period.clone()),
    };

    let output_ports = RmsTrendOutputPorts {
        windowed_downsampled_data: refresh_period_downsampler.output.clone(),
    };

    Pipeline::new(
        vec![
            Box::new(buffer_size),
            Box::new(buffer_size_to_usize),
            Box::new(buffer),
            Box::new(refresh_factor),
            Box::new(refresh_factor_to_usize),
            Box::new(refresh_period_downsampler),
            Box::new(chart),
        ],
        input_ports,
        output_ports,
    )
}

pub struct RmsTrend {
    data: Arc<RwLock<Vec<[f64; 2]>>>,

    prev_chart_size: f64,
}

impl RmsTrend {
    pub fn new(data: Arc<RwLock<Vec<[f64; 2]>>>) -> Self {
        Self {
            data,
            prev_chart_size: f64::NEG_INFINITY,
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        chart_size: ChartSize,
        unit: VoltageUnit,
        precision: Precision,
    ) {
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 10.0;

            ui.label(RichText::new("RMS Trend").size(20.0).strong());

            let chart_size = chart_size as f64;

            let mut plot = Plot::new("RmsTrend")
                .auto_bounds(Vec2b::new(false, true))
                .y_axis_label("Voltage")
                .x_axis_label("Time")
                .allow_boxed_zoom(false)
                .allow_drag(false)
                .allow_zoom(false)
                .allow_scroll(false)
                .label_formatter(|_, _| "".to_owned())
                .coordinates_formatter(
                    egui_plot::Corner::LeftTop,
                    coordinates_formatter(unit, precision),
                )
                .x_axis_formatter(|grid_mark, range| {
                    format!(
                        "{:.precision$} s",
                        grid_mark.value,
                        precision = calculate_precision(range)
                    )
                })
                .y_axis_formatter(|grid_mark, range| {
                    unit.apply_unit_with_precision(grid_mark.value, calculate_precision(range))
                })
                .include_y(0.0)
                .include_x(0.0)
                .include_x(-chart_size);

            // We need to check if the chart size has changed to reset the plot, otherwise the
            // plot will not update the chart size.
            if (self.prev_chart_size - chart_size).abs() > f64::EPSILON {
                plot = plot.reset();
                self.prev_chart_size = chart_size;
            }

            plot.show(ui, |plot_ui| {
                plot_ui.line(self.signal());
            });
        });
    }

    fn signal(&self) -> Line {
        let plot_points = PlotPoints::from_iter(self.data.read().unwrap().clone());

        Line::new(plot_points)
            .color(Color32::LIGHT_BLUE)
            .name("Signal")
    }
}
