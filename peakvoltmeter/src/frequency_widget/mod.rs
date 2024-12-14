mod chart;

use crate::{
    application::{calculate_precision, Precision},
    settings::{ChartSize, FftSize, RefreshPeriod, SampleRate},
    DARK_GRAY,
};
use chart::Chart;
use conductor::{core::pipeline::Pipeline, prelude::NodeConfigInputPort};
use core::f64;
use eframe::egui::Frame;
use egui::{Align, Color32, Layout, RichText, Rounding, Vec2b};
use egui_plot::{CoordinatesFormatter, Line, Plot, PlotPoints};
use std::sync::{Arc, RwLock};

pub struct FrequencyWidgetInputPorts {
    pub fft_input: NodeConfigInputPort<Vec<f64>>,
    pub chart_size: NodeConfigInputPort<ChartSize>,
    pub sample_rate: NodeConfigInputPort<SampleRate>,
    pub fft_size: NodeConfigInputPort<FftSize>,
    pub refresh_period: NodeConfigInputPort<RefreshPeriod>,
}

pub fn frequency_widget(
    data: Arc<RwLock<Vec<[f64; 2]>>>,
) -> Pipeline<FrequencyWidgetInputPorts, ()> {
    let chart = Chart::new(data);

    let input_ports = FrequencyWidgetInputPorts {
        fft_input: chart.fft_input.clone(),
        chart_size: chart.chart_size.clone(),
        sample_rate: chart.sample_rate.clone(),
        fft_size: chart.fft_size.clone(),
        refresh_period: chart.refresh_period.clone(),
    };

    Pipeline::new(vec![Box::new(chart)], input_ports, ())
}

pub struct FrequencyWidget {
    data: Arc<RwLock<Vec<[f64; 2]>>>,

    prev_chart_size: f64,
}

impl FrequencyWidget {
    pub fn new(data: Arc<RwLock<Vec<[f64; 2]>>>) -> Self {
        Self {
            data,
            prev_chart_size: f64::NEG_INFINITY,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, chart_size: ChartSize, precision: Precision) {
        let frame = Frame::default()
            .inner_margin(10.0)
            .fill(DARK_GRAY)
            .rounding(Rounding::same(10.0));

        frame.show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;

            ui.style_mut().visuals.extreme_bg_color = DARK_GRAY;
            ui.style_mut().visuals.override_text_color = Some(Color32::WHITE);

            ui.label(RichText::new("Frequency").size(16.0));

            let last_value = self
                .data
                .read()
                .unwrap()
                .last()
                .map(|v| v[1])
                .unwrap_or(0.0);

            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                ui.label(
                    RichText::new(format!(
                        "{:.precision$} Hz",
                        last_value,
                        precision = precision
                    ))
                    .size(30.0)
                    .strong(),
                );
            });

            let chart_size = chart_size as f64;

            let coordinates_formatter = CoordinatesFormatter::new(|plot_point, _| {
                let x = plot_point.x;
                let y = plot_point.y;

                format!(
                    "x = {:.precision$} s\ny = {:.precision$} Hz",
                    x,
                    y,
                    precision = precision
                )
            });

            let mut plot = Plot::new("Frequency Widget")
                .auto_bounds(Vec2b::new(false, true))
                .y_axis_label("Frequency")
                .x_axis_label("Time")
                .allow_boxed_zoom(false)
                .allow_drag(false)
                .allow_zoom(false)
                .allow_scroll(false)
                .label_formatter(|_, _| "".to_owned())
                .coordinates_formatter(egui_plot::Corner::LeftTop, coordinates_formatter)
                .x_axis_formatter(|grid_mark, range| {
                    format!(
                        "{:.precision$} s",
                        grid_mark.value,
                        precision = calculate_precision(range)
                    )
                })
                .y_axis_formatter(|grid_mark, range| {
                    format!(
                        "{:.precision$} Hz",
                        grid_mark.value,
                        precision = calculate_precision(range)
                    )
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
