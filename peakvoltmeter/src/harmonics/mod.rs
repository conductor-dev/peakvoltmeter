mod chart;

use crate::{PeakVoltmeterPacket, FFT_SIZE, SAMPLE_RATE};
use chart::Chart;
use conductor::{core::pipeline::Pipeline, prelude::*};
use egui::{Color32, RichText, Vec2b};
use egui_plot::{Line, Plot, PlotPoints};
use rustfft::num_complex::Complex;
use std::sync::{Arc, RwLock};

pub fn harmonics(
    data: Arc<RwLock<Vec<f64>>>,
) -> Pipeline<NodeConfigInputPort<PeakVoltmeterPacket>, ()> {
    let into_f32 = Intoer::<_, f32>::new();

    let fft_buffer = Buffer::new(false);

    let hann_window = Window::new(WindowType::Hann);

    let fft = FFT::new();
    let downsampler = Downsampler::new(600);
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

    fft_buffer.size.set_initial(FFT_SIZE);

    into_f32.output.connect(&fft_buffer.input);

    fft_buffer.output.connect(&downsampler.input);

    downsampler.output.connect(&hann_window.input);

    hann_window.output.connect(&fft.input);

    fft.output.connect(&lambda.input);

    lambda.output.connect(&chart.input);

    let input = into_f32.input.clone();

    Pipeline::new(
        vec![
            Box::new(into_f32),
            Box::new(fft_buffer),
            Box::new(hann_window),
            Box::new(fft),
            Box::new(downsampler),
            Box::new(lambda),
            Box::new(chart),
        ],
        input,
        (),
    )
}

pub struct Harmonics {
    data: Arc<RwLock<Vec<f64>>>,
}

impl Harmonics {
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

                ui.label(RichText::new("Harmonics").size(20.0).strong());

                let plot = Plot::new("Harmonics")
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
                    .include_x(Self::y_to_hz(FFT_SIZE as f64 / 2.0));

                plot.show(ui, |plot_ui| {
                    plot_ui.line(self.signal());
                });
            },
        );
    }

    fn y_to_hz(y: f64) -> f64 {
        y * (SAMPLE_RATE as f64 / FFT_SIZE as f64)
    }

    fn signal(&self) -> Line {
        let plot_points = PlotPoints::from_iter(
            self.data
                .read()
                .unwrap()
                .clone()
                .into_iter()
                .enumerate()
                .map(|(i, v)| [Self::y_to_hz(i as f64), v]),
        );

        Line::new(plot_points)
            .color(Color32::LIGHT_BLUE)
            .name("Signal")
    }
}
