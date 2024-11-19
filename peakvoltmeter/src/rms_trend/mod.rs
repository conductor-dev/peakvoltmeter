mod chart;

use crate::{
    settings::{ChartSize, RefreshPeriod, RmsWindow, SampleRate},
    PeakVoltmeterPacket,
};
use chart::Chart;
use conductor::{core::pipeline::Pipeline, prelude::*};
use egui::{Color32, RichText, Vec2b};
use egui_plot::{Line, Plot, PlotPoints};
use std::sync::{Arc, RwLock};

pub struct RmsTrendInputPorts {
    pub data: NodeConfigInputPort<PeakVoltmeterPacket>,
    pub sample_rate: NodeConfigInputPort<SampleRate>,
    pub window: NodeConfigInputPort<RmsWindow>,
    pub chart_size: NodeConfigInputPort<ChartSize>,
    pub rms_refresh_period: (
        NodeConfigInputPort<RefreshPeriod>,
        NodeConfigInputPort<RefreshPeriod>,
    ),
}

pub struct RmsTrendOutputPorts {
    pub windowed_downsampled_data: NodeConfigOutputPort<Vec<i32>>,
}

pub fn rms_trend(
    data: Arc<RwLock<Vec<[f64; 2]>>>,
) -> Pipeline<RmsTrendInputPorts, RmsTrendOutputPorts> {
    let into_i32 = Intoer::<_, i32>::new();

    let sample_rate_to_f32 = Lambdaer::new(|value: usize| value as f32);

    let buffer_size = Multiplier::new();

    let buffer_size_to_usize = Lambdaer::new(|value: f32| value as usize);

    let buffer = Buffer::new(true);

    let refresh_factor = Multiplier::new();

    let refresh_factor_to_usize = Lambdaer::new(|value: f32| value as usize);

    let refresh_period_downsampler = Downsampler::new();

    let chart = Chart::new(data);

    into_i32.output.connect(&buffer.input);

    sample_rate_to_f32.output.connect(&buffer_size.input2);

    buffer_size.output.connect(&buffer_size_to_usize.input);

    buffer_size_to_usize.output.connect(&buffer.size);

    sample_rate_to_f32.output.connect(&refresh_factor.input2);

    refresh_factor
        .output
        .connect(&refresh_factor_to_usize.input);

    refresh_factor_to_usize
        .output
        .connect(&refresh_period_downsampler.factor);

    buffer.output.connect(&refresh_period_downsampler.input);

    refresh_period_downsampler.output.connect(&chart.input);

    let input_ports = RmsTrendInputPorts {
        data: into_i32.input.clone(),
        sample_rate: sample_rate_to_f32.input.clone(),
        window: buffer_size.input1.clone(),
        chart_size: chart.chart_size.clone(),
        rms_refresh_period: (refresh_factor.input1.clone(), chart.refresh_period.clone()),
    };

    let output_ports = RmsTrendOutputPorts {
        windowed_downsampled_data: refresh_period_downsampler.output.clone(),
    };

    Pipeline::new(
        vec![
            Box::new(into_i32),
            Box::new(sample_rate_to_f32),
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

    pub fn ui(&mut self, ui: &mut egui::Ui, chart_size: usize) {
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
