mod chart;

use crate::{
    application::{calculate_precision, Precision, VoltageUnit},
    coordinates_formatter,
    settings::{ChartSize, RefreshPeriod},
    DARK_GRAY,
};
use chart::Chart;
use conductor::{core::pipeline::Pipeline, prelude::NodeConfigInputPort};
use core::f64;
use eframe::egui::Frame;
use egui::{Align, Color32, Layout, RichText, Rounding, Vec2b};
use egui_plot::{Line, Plot, PlotPoints};
use std::sync::{Arc, RwLock};

pub struct PeakSqrtInputPorts {
    pub windowed_downsampled_data: NodeConfigInputPort<Vec<f32>>,
    pub chart_size: NodeConfigInputPort<ChartSize>,
    pub refresh_period: NodeConfigInputPort<RefreshPeriod>,
}

pub fn peak_sqrt(data: Arc<RwLock<Vec<[f64; 2]>>>) -> Pipeline<PeakSqrtInputPorts, ()> {
    let chart = Chart::new(data);

    let input_ports = PeakSqrtInputPorts {
        windowed_downsampled_data: chart.windowed_downsampled_data.clone(),
        chart_size: chart.chart_size.clone(),
        refresh_period: chart.refresh_period.clone(),
    };

    Pipeline::new(vec![Box::new(chart)], input_ports, ())
}

pub struct PeakSqrtChart {
    data: Arc<RwLock<Vec<[f64; 2]>>>,

    prev_chart_size: f64,
}

impl PeakSqrtChart {
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
        let available_size = ui.available_size();

        ui.allocate_ui_with_layout(
            egui::vec2(available_size.x, available_size.y / 3.0),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                let frame = Frame::default()
                    .inner_margin(10.0)
                    .fill(DARK_GRAY)
                    .rounding(Rounding::same(10.0));

                frame.show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 10.0;

                    ui.style_mut().visuals.extreme_bg_color = DARK_GRAY;
                    ui.style_mut().visuals.override_text_color = Some(Color32::WHITE);

                    ui.label(RichText::new("Vp / √2").size(16.0));

                    let last_value = self
                        .data
                        .read()
                        .unwrap()
                        .last()
                        .map(|v| v[1])
                        .unwrap_or(0.0);

                    ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                        ui.label(
                            RichText::new(unit.apply_unit_with_precision(last_value, precision))
                                .size(30.0)
                                .strong(),
                        );
                    });

                    let chart_size = chart_size as f64;

                    let mut plot = Plot::new("Peak Sqrt Widget")
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
                            unit.apply_unit_with_precision(
                                grid_mark.value,
                                calculate_precision(range),
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
