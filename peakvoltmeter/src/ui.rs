use crate::trigger::RisingEdgeTriggerExt;
use conductor::{
    core::{NodeConfig, NodeRunner},
    prelude::{CircularBuffer, NodeConfigInputPort, NodeRunnerInputPort},
};
use egui::Color32;
use egui_plot::{Legend, Line, MarkerShape, Plot, PlotPoints, Points};
use std::sync::{Arc, RwLock};

struct UiRunner {
    input: NodeRunnerInputPort<f32>,
    buffer: Arc<RwLock<CircularBuffer<f32>>>,
}

impl NodeRunner for UiRunner {
    fn run(self: Box<Self>) {
        loop {
            let value = self.input.recv().unwrap();

            self.buffer.write().unwrap().push(value);
        }
    }
}

pub struct Ui {
    pub(crate) input: NodeConfigInputPort<f32>,
    buffer: Arc<RwLock<CircularBuffer<f32>>>,
}

impl Ui {
    pub fn new(buffer: Arc<RwLock<CircularBuffer<f32>>>) -> Self {
        Self {
            input: NodeConfigInputPort::new(),
            buffer,
        }
    }
}

impl NodeConfig for Ui {
    fn into_runner(self: Box<Self>) -> Box<dyn NodeRunner + Send> {
        Box::new(UiRunner {
            input: self.input.into(),
            buffer: self.buffer,
        })
    }
}

pub(crate) struct UiApp {
    axes: bool,
    grid: bool,
    sample_markers: bool,
    sample_marker_radius: f32,
    trigger_enabled: bool,
    trigger_threshold: f64,
    trigger_window: usize,
    data: Arc<RwLock<CircularBuffer<f32>>>,
}

impl UiApp {
    pub(crate) fn new(data: Arc<RwLock<CircularBuffer<f32>>>) -> Self {
        let buffer_size = data.read().unwrap().size();

        Self {
            axes: true,
            grid: true,
            sample_markers: false,
            sample_marker_radius: 5.0,
            trigger_enabled: false,
            trigger_threshold: 0.0,
            trigger_window: buffer_size / 2,
            data,
        }
    }

    fn options_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.checkbox(&mut self.axes, "Show axes");
                ui.checkbox(&mut self.grid, "Show grid");
                ui.checkbox(&mut self.sample_markers, "Show sample markers");
                ui.add(
                    egui::DragValue::new(&mut self.sample_marker_radius)
                        .speed(0.1)
                        .range(0.5..=20.0),
                )
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.checkbox(&mut self.trigger_enabled, "Trigger");
                ui.add(egui::DragValue::new(&mut self.trigger_threshold).speed(0.01));
                ui.add(
                    egui::Slider::new(
                        &mut self.trigger_window,
                        1..=self.data.read().unwrap().size(),
                    )
                    .text("Window")
                    .clamping(egui::SliderClamping::Always),
                );
            });
        });
    }

    fn _plot_points(&self) -> PlotPoints {
        let mut points = self
            .data
            .read()
            .unwrap()
            .iter()
            .map(|v| *v as f64)
            .collect::<Vec<_>>();

        if self.trigger_enabled {
            points = points
                .into_iter()
                .rising_edge_trigger(self.trigger_threshold)
                .take(self.trigger_window)
                .collect();
        }

        PlotPoints::from_iter(points.into_iter().enumerate().map(|(i, v)| [i as f64, v]))
    }

    fn signal(&self) -> Line {
        Line::new(self._plot_points())
            .color(Color32::LIGHT_RED)
            .name("Signal")
    }

    fn sample_markers(&self) -> Points {
        Points::new(self._plot_points())
            .shape(MarkerShape::Circle)
            .color(Color32::RED)
            .radius(self.sample_marker_radius)
            .name("Sample markers")
    }
}

impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.options_ui(ui);

            ui.separator();

            ui.ctx().request_repaint();

            let plot = Plot::new("Plot")
                .legend(Legend::default())
                .show_axes(self.axes)
                .show_grid(self.grid)
                .x_axis_label("Samples");

            plot.show(ui, |plot_ui| {
                plot_ui.line(self.signal());
                if self.sample_markers {
                    plot_ui.points(self.sample_markers());
                }
            })
        });
    }
}
