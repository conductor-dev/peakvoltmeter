mod chart;

use crate::settings::{FftSize, SampleRate};
use chart::Chart;
use conductor::{core::pipeline::Pipeline, prelude::*};
use egui::{Color32, RichText, Vec2b};
use egui_plot::{Line, Plot, PlotPoints};
use rustfft::num_complex::Complex;
use std::sync::{Arc, RwLock};

pub const DOWNSAMPLING_FACTOR: usize = 600;

pub struct HarmonicsInputPorts {
    pub data: NodeConfigInputPort<f32>,
    pub fft_size: (NodeConfigInputPort<FftSize>, NodeConfigInputPort<FftSize>),
    pub sample_rate: NodeConfigInputPort<SampleRate>,
}

pub struct HarmonicsOutputPorts {
    pub fft_output: NodeConfigOutputPort<Vec<f64>>,
}

pub fn harmonics(
    data: Arc<RwLock<Vec<[f64; 2]>>>,
) -> Pipeline<HarmonicsInputPorts, HarmonicsOutputPorts> {
    let fft_buffer = Buffer::new(false);

    let hann_window = Window::new(WindowType::Hamming);

    let fft = FFT::new();

    let downsampler = Downsampler::new();
    downsampler.factor.set_initial(DOWNSAMPLING_FACTOR);

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

    fft_buffer.output.connect(&downsampler.input);

    downsampler.output.connect(&hann_window.input);

    hann_window.output.connect(&fft.input);

    fft.output.connect(&lambda.input);

    lambda.output.connect(&chart.input);

    let input_ports = HarmonicsInputPorts {
        data: fft_buffer.input.clone(),
        fft_size: (fft_buffer.size.clone(), chart.fft_size.clone()),
        sample_rate: chart.sample_rate.clone(),
    };

    let output_ports = HarmonicsOutputPorts {
        fft_output: lambda.output.clone(),
    };

    Pipeline::new(
        vec![
            Box::new(fft_buffer),
            Box::new(hann_window),
            Box::new(fft),
            Box::new(downsampler),
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

    pub fn ui(&mut self, ui: &mut egui::Ui, sample_rate: usize) {
        let available_size = ui.available_size();

        ui.allocate_ui_with_layout(
            egui::vec2(available_size.x, available_size.y / 2.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.y = 10.0;

                ui.label(RichText::new("Harmonics").size(20.0).strong());

                let x_bound = sample_rate as f64 / 2.0;

                let mut plot = Plot::new("Harmonics")
                    .auto_bounds(Vec2b::FALSE)
                    .y_axis_label("Signal Strength (dBV)")
                    .x_axis_label("Frequency (Hz)")
                    .allow_boxed_zoom(false)
                    .allow_drag(false)
                    .allow_zoom(false)
                    .allow_scroll(false)
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
