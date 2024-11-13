use crate::{FFT_SIZE, SAMPLE_RATE};
use conductor::{
    core::{NodeConfig, NodeRunner},
    prelude::{NodeConfigInputPort, NodeRunnerInputPort},
};
use egui::{Color32, RichText, Vec2b};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use std::sync::{Arc, RwLock};

struct HarmonicsRunner {
    data: Arc<RwLock<Vec<f64>>>,

    input: NodeRunnerInputPort<Vec<f64>>,
}

impl NodeRunner for HarmonicsRunner {
    fn run(self: Box<Self>) {
        loop {
            let input = self.input.recv().unwrap();

            *self.data.write().unwrap() = input;
        }
    }
}

pub struct Harmonics {
    data: Arc<RwLock<Vec<f64>>>,

    pub input: NodeConfigInputPort<Vec<f64>>,
}

impl Harmonics {
    pub fn new(data: Arc<RwLock<Vec<f64>>>) -> Self {
        Self {
            data,

            input: NodeConfigInputPort::new(),
        }
    }
}

impl NodeConfig for Harmonics {
    fn into_runner(self: Box<Self>) -> Box<dyn NodeRunner + Send> {
        Box::new(HarmonicsRunner {
            data: self.data,

            input: self.input.into(),
        })
    }
}

pub struct HarmonicsUi {
    data: Arc<RwLock<Vec<f64>>>,
}

impl HarmonicsUi {
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
                    .legend(Legend::default())
                    .y_axis_label("Signal Strength (dBV)")
                    .x_axis_label("Frequency (Hz)")
                    .allow_boxed_zoom(false)
                    .allow_drag(false)
                    .allow_zoom(false)
                    .allow_scroll(false)
                    .include_y(0.0)
                    .include_y(-150)
                    .include_x(0.0)
                    .include_x(FFT_SIZE as f64 / 2.0);

                plot.show(ui, |plot_ui| {
                    plot_ui.line(self.signal());
                });
            },
        );
    }

    fn signal(&self) -> Line {
        let plot_points = PlotPoints::from_iter(
            self.data
                .read()
                .unwrap()
                .clone()
                .into_iter()
                .enumerate()
                .map(|(i, v)| [i as f64 * (SAMPLE_RATE as f64 / FFT_SIZE as f64), v]),
        );

        Line::new(plot_points)
            .color(Color32::LIGHT_RED)
            .name("Signal")
    }
}
