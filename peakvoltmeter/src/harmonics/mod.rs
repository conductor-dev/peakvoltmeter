mod chart;

use crate::{
    application::{calculate_precision, Precision},
    settings::{FftSize, RefreshPeriod, SampleRate},
};
use chart::Chart;
use conductor::{core::pipeline::Pipeline, prelude::*};
use egui::{Color32, RichText, Vec2b};
use egui_plot::{CoordinatesFormatter, Line, Plot, PlotPoints};
use rustfft::num_complex::Complex;
use std::sync::{Arc, RwLock};

pub struct HarmonicsInputPorts {
    pub data: NodeConfigInputPort<f32>,
    pub fft_size: (NodeConfigInputPort<FftSize>, NodeConfigInputPort<FftSize>),
    pub sample_rate: (
        NodeConfigInputPort<SampleRate>,
        NodeConfigInputPort<SampleRate>,
    ),
    pub refresh_period: NodeConfigInputPort<RefreshPeriod>,
}

pub struct HarmonicsOutputPorts {
    pub fft_output: NodeConfigOutputPort<Vec<f64>>,
}

pub fn harmonics(
    data: Arc<RwLock<Vec<[f64; 2]>>>,
) -> Pipeline<HarmonicsInputPorts, HarmonicsOutputPorts> {
    let fft_buffer = Buffer::new(false);

    let refresh_factor = Multiplier::new();

    let refresh_factor_to_usize = Lambdaer::new(|value: f32| value as usize);

    let refresh_period_downsampler = Downsampler::new();

    let hann_window = Window::new(WindowType::Hamming);

    let fft = FFT::new();

    let lambda = Lambdaer::new(|fft: Vec<Complex<f32>>| {
        let length = fft.len();

        let fft = fft
            .into_iter()
            .take(length / 2)
            .map(|value| value.norm() as f64)
            .collect::<Vec<_>>();

        let max = fft.iter().cloned().fold(f64::MIN, f64::max);

        fft.into_iter()
            .map(|value| 20.0 * (value / max).log10())
            .collect()
    });

    let chart = Chart::new(data);

    refresh_factor
        .output
        .connect(&refresh_factor_to_usize.input);

    refresh_factor_to_usize
        .output
        .connect(&refresh_period_downsampler.factor);

    fft_buffer.output.connect(&refresh_period_downsampler.input);

    refresh_period_downsampler
        .output
        .connect(&hann_window.input);

    hann_window.output.connect(&fft.input);

    fft.output.connect(&lambda.input);

    lambda.output.connect(&chart.input);

    let input_ports = HarmonicsInputPorts {
        data: fft_buffer.input.clone(),
        fft_size: (fft_buffer.size.clone(), chart.fft_size.clone()),
        sample_rate: (chart.sample_rate.clone(), refresh_factor.input2.clone()),
        refresh_period: refresh_factor.input1.clone(),
    };

    let output_ports = HarmonicsOutputPorts {
        fft_output: lambda.output.clone(),
    };

    Pipeline::new(
        vec![
            Box::new(fft_buffer),
            Box::new(refresh_factor),
            Box::new(refresh_factor_to_usize),
            Box::new(refresh_period_downsampler),
            Box::new(hann_window),
            Box::new(fft),
            Box::new(lambda),
            Box::new(chart),
        ],
        input_ports,
        output_ports,
    )
}

pub struct Harmonics {
    data: Arc<RwLock<Vec<[f64; 2]>>>,

    prev_x_bound: f64,
}

impl Harmonics {
    pub fn new(data: Arc<RwLock<Vec<[f64; 2]>>>) -> Self {
        Self {
            data,
            prev_x_bound: f64::NEG_INFINITY,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, sample_rate: SampleRate, precision: Precision) {
        let available_size = ui.available_size();

        ui.allocate_ui_with_layout(
            egui::vec2(available_size.x, available_size.y / 2.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.y = 10.0;

                ui.label(RichText::new("Harmonics").size(20.0).strong());

                let x_bound = sample_rate as f64 / 2.0;

                let coordinates_formatter = CoordinatesFormatter::new(|plot_point, _| {
                    let x = plot_point.x;
                    let y = plot_point.y;

                    format!(
                        "x = {:.precision$} Hz\ny = {:.precision$} dBV",
                        x,
                        y,
                        precision = precision
                    )
                });

                let mut plot = Plot::new("Harmonics")
                    .auto_bounds(Vec2b::FALSE)
                    .y_axis_label("Signal Strength (dBV)")
                    .x_axis_label("Frequency (Hz)")
                    .allow_boxed_zoom(false)
                    .allow_drag(false)
                    .allow_zoom(false)
                    .allow_scroll(false)
                    .label_formatter(|_, _| "".to_owned())
                    .coordinates_formatter(egui_plot::Corner::LeftTop, coordinates_formatter)
                    .x_axis_formatter(|grid_mark, range| {
                        format!(
                            "{:.precision$} Hz",
                            grid_mark.value,
                            precision = calculate_precision(range)
                        )
                    })
                    .y_axis_formatter(|grid_mark, range| {
                        format!(
                            "{:.precision$} dBV",
                            grid_mark.value,
                            precision = calculate_precision(range)
                        )
                    })
                    .include_y(0.0)
                    .include_y(-200)
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
