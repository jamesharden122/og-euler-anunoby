use crate::ops::MyMatrix;
use chrono::{TimeZone, Utc};
use dioxus::prelude::*;
use tracing::info;

#[component]
pub fn ScatterPlot(props: MyMatrix) -> Element {
    let width = 470.0;
    let height = 500.0;
    let y_padding = 70.0;
    let x_padding = y_padding / 1.5;

    let max_points = 300;

    // ts_recv (column 3) = x-axis, size (column 1) = y-axis
    let mut combined: Vec<(f64, f64)> = props
        .data
        .column(0)
        .iter()
        .copied()
        .zip(props.data.column(1).iter().copied())
        .take(max_points)
        .collect();

    combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let x_values: Vec<f64> = combined.iter().map(|(x, _)| *x).collect();
    let y_values: Vec<f64> = combined.iter().map(|(_, y)| *y).collect();

    if x_values.is_empty() || y_values.is_empty() {
        return rsx!(div { "No data available" });
    }

    let x_min = *x_values
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let x_max = *x_values
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let y_min = *y_values
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let y_max = *y_values
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    let scale_x = |x: f64| ((x - x_min) / (x_max - x_min) * (width - 2.0 * x_padding)) + x_padding;
    let scale_y =
        |y: f64| height - ((y - y_min) / (y_max - y_min) * (height - 2.0 * y_padding)) - y_padding;

    let x_ticks = 6;
    let y_ticks = 20;

    let x_tick_values: Vec<f64> = (0..x_ticks)
        .map(|i| x_min + (i as f64 / (x_ticks - 1) as f64) * (x_max - x_min))
        .collect();

    let y_tick_values: Vec<f64> = (0..y_ticks)
        .map(|i| y_min + (i as f64 / (y_ticks - 1) as f64) * (y_max - y_min))
        .collect();

    let x_tick_elements: Vec<_> = x_tick_values
        .iter()
        .map(|&xv| {
            let x_pos = scale_x(xv);
            let label = format!("{:.2}", xv); // format as price

            rsx!(
                line {
                    x1: "{x_pos}",
                    y1: "{height - y_padding}",
                    x2: "{x_pos}",
                    y2: "{height - y_padding + 5.0}",
                    stroke: "black",
                    stroke_width: "1"
                },
                text {
                    x: "{x_pos}",
                    y: "{height - y_padding + 20.0}",
                    font_size: "11",
                    font_family: "Georgia",
                    font_weight: "700",
                    text_anchor: "middle",
                    fill: "black",
                    "{label}"
                }
            )
        })
        .collect();

    let y_tick_elements: Vec<_> = y_tick_values
        .iter()
        .map(|&yv| {
            let y_pos = scale_y(yv);
            let label = format!("{}", yv.round() as i64);
            rsx!(
                // Horizontal grid line
                line {
                    x1: "{x_padding}",
                    y1: "{y_pos}",
                    x2: "{width - x_padding}",
                    y2: "{y_pos}",
                    stroke: "#00bcd4",
                    stroke_width: "0.5",
                    stroke_dasharray: "3 3",
                    stroke_opacity: "0.25"
                },

                // Y-axis tick mark
                line {
                    x1: "{x_padding - 6.0}",
                    y1: "{y_pos}",
                    x2: "{x_padding}",
                    y2: "{y_pos}",
                    stroke: "#90A4AE",
                    stroke_width: "1"
                },

                // Label in dark color
                text {
                    x: "{x_padding - 10.0}",
                    y: "{y_pos + 4.0}",
                    font_size: "11",
                    font_family: "Georgia",
                    font_weight: "700",
                    text_anchor: "end",
                    fill: "#222",  // dark gray text for readability on light strip
                    "{label}"
                }
            )
        })
        .collect();

    // Circles instead of polyline
    let circles: Vec<_> = x_values
        .iter()
        .zip(y_values.iter())
        .enumerate()
        .map(|(i, (&x, &y))| {
            let scaled_x = scale_x(x);
            let scaled_y = scale_y(y);
            let r = 3.0 + (y / y_max * 4.0);

            rsx!(circle {
                key: "{i}",
                cx: "{scaled_x}",
                cy: "{scaled_y}",
                r: "{r}",
                fill: "#43a047",
                stroke: "black",
                stroke_width: "0.3",
                opacity: "0.7"
            })
        })
        .collect();

    rsx! {
        svg {
            width: "{width}",
            height: "{height}",
            style: "background-color: white",  // Updated background
            rect {
                x: "{x_padding}",
                y: "{y_padding / 4.0}",
                width: "{width - x_padding * 1.25}",
                height: "{height - y_padding - y_padding / 4.0}",
                fill: "rgba(78, 78, 205, 0.12)", // faint overlay
            },
            // Y-axis line
            line {
                x1: "{x_padding}",
                y1: "{y_padding/4.0}",
                x2: "{x_padding}",
                y2: "{height - y_padding}",
                stroke: "#90A4AE", // gray-blue
                stroke_width: "1"
            },

            // X-axis line
            line {
                x1: "{x_padding}",
                y1: "{height - y_padding}",
                x2: "{width - x_padding}",
                y2: "{height - y_padding}",
                stroke: "#90A4AE",
                stroke_width: "1"
            },

            // Tick marks and labels
            { x_tick_elements.into_iter() },
            { y_tick_elements.into_iter() },

            // Scatter points
            { circles.into_iter() },

            // Y-axis label
            text {
                x: "10.0",
                y: "{height / 2.0}",
                transform: "rotate(-90, 10.0, {height / 2.0})",
                font_size: "14",
                font_family: "Georgia",
                font_weight: "700",
                text_anchor: "middle",
                fill: "black",
                "Size"
            },

            // X-axis label
            text {
                x: "{width / 2.0}",
                y: "{height - 5.0}",
                font_size: "14",
                font_family: "Georgia",
                font_weight: "700",
                text_anchor: "middle",
                fill: "black",
                "Price"
            }
        }
    }
}
