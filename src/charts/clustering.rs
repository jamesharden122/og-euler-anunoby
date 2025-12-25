use crate::ops::{multi_type_mat::MyMmMatrix, MyMatrix};
use dioxus::{html::optgroup::label, prelude::*};
use ndarray::Array2;

#[derive(Props, Clone, Debug, PartialEq)]
pub struct NmsPca {
    pub components: Array2<f64>,
    pub records: Array2<f64>,
    pub labels: Option<Array2<f64>>,
    pub nms: Vec<String>,
}

#[component]
pub fn PcaChart(pca_nms: NmsPca) -> Element {
    let view_w = 400.0_f64;
    let view_h = 300.0_f64;
    let axis_left = 0.09 * view_w;
    let axis_right = 0.95 * view_w;
    let axis_top = 0.05 * view_h;
    let axis_bottom = 0.75 * view_h;
    let mut labels: Vec<f64> = Vec::new();
    if let Some(lbls) = pca_nms.labels {
        (labels, _) = lbls.into_raw_vec_and_offset();
    }
    tracing::debug!("{:?}", &labels);
    let record = pca_nms.records;
    let comp1: Vec<f64> = record.column(0).to_owned().to_vec();
    let comp2: Vec<f64> = record.column(1).to_owned().to_vec();

    let x_min = comp1
        .iter()
        .copied()
        .min_by(|a, b| a.total_cmp(b))
        .unwrap_or(0.0);
    let x_max = comp1
        .iter()
        .copied()
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(1.0);
    let y_min = comp2
        .iter()
        .copied()
        .min_by(|a, b| a.total_cmp(b))
        .unwrap_or(0.0);
    let y_max = comp2
        .iter()
        .copied()
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(1.0);

    let denom_x = (x_max - x_min).abs().max(1e-12);
    let denom_y = (y_max - y_min).abs().max(1e-12);
    let scale_x = |x: f64| axis_left + ((x - x_min) / denom_x) * (axis_right - axis_left);
    let scale_y = |y: f64| axis_bottom - ((y - y_min) / denom_y) * (axis_bottom - axis_top);

    let x_ticks = 6;
    let y_ticks = 20;
    let tick_at = |mn: f64, mx: f64, i: usize, n: usize| {
        if n <= 1 {
            return mn;
        }
        mn + (i as f64 / (n as f64 - 1.0)) * (mx - mn)
    };
    let x_tick_values: Vec<f64> = (0..x_ticks)
        .map(|i| tick_at(x_min, x_max, i, x_ticks))
        .collect();

    let y_tick_values: Vec<f64> = (0..y_ticks)
        .map(|i| tick_at(y_min, y_max, i, y_ticks))
        .collect();

    let x_tick_elements: Vec<_> = x_tick_values
        .iter()
        .map(|&xv| {
            let x_pos = scale_x(xv);
            let tick_label = format!("{:.2}", xv); // format as price

            rsx!(
                line {
                    x1: "{x_pos}", y1: "{axis_bottom}", x2: "{x_pos}",
                    y2: "{axis_bottom + 5.0}",  stroke: "black", stroke_width: "1"
                },
                text {
                    x: "{x_pos}",  y: "{axis_bottom + 20.0}", font_size: "7",
                    font_family: "Georgia", font_weight: "700", text_anchor: "middle", fill: "white",
                    "{tick_label}"
                }
            )
        })
        .collect();

    let y_tick_elements: Vec<_> = y_tick_values
            .iter()
            .map(|&yv| {
                let y_pos = scale_y(yv);
                let tick_label = format!("{:.2}", yv);
                rsx!(
                    // Horizontal grid line
                    line {
                        x1: "{axis_left}", y1: "{y_pos}", x2: "{axis_right}", y2: "{y_pos}",
                        stroke: "#00bcd4", stroke_width: "0.5", stroke_dasharray: "3 3", stroke_opacity: "0.25"
                    },
                    // Y-axis tick mark
                    line {
                        x1: "{axis_left - 6.0}", y1: "{y_pos}", x2: "{axis_left}",
                        y2: "{y_pos}", stroke: "#90A4AE", stroke_width: "1"
                    },
                    // Label in dark color
                    text {
                        x: "{axis_left - 10.0}", y: "{y_pos + 4.0}", font_size: "7",
                        font_family: "Georgia", font_weight: "700", text_anchor: "end", fill: "white",
                        "{tick_label}"
                    }
                )
            })
            .collect();

    fn color_circle(group: &f64) -> &'static str {
        // Treat group as an integer label (e.g., 0.0, 1.0, 2.0...)
        match group.round() as i64 {
            0 => "#4CAF50", // green
            1 => "#2196F3", // blue
            2 => "#FF9800", // orange
            3 => "#E91E63", // pink
            4 => "#9C27B0", // purple
            5 => "#00BCD4", // cyan
            6 => "#FFC107", // amber
            7 => "#F44336", // red
            _ => "#9E9E9E", // gray (fallback)
        }
    }

    let circles: Vec<_> = comp1
        .iter()
        .zip(comp2.iter().zip(labels.iter()))
        .enumerate()
        .map(|(i, (&x, (&y, l)))| {
            let scaled_x = scale_x(x);
            let scaled_y = scale_y(y);
            let r = 3.0;
            let cc = color_circle(l);
            rsx!(
            g {
                key: "{i}",
                class: "pt",
                circle {
                    key: "{i}",
                    cx: "{scaled_x}",
                    cy: "{scaled_y}",
                    r: "{r}",
                    fill: "{cc}",
                    stroke: "black",
                    stroke_width: "0.3",
                    opacity: "0.7",
                }
                text {
                    class: "pt-label",
                    x: "{scaled_x + 6.0}",
                    y: "{scaled_y - 6.0}",
                    font_size: "9",
                    font_family: "Georgia",
                    fill: "#111",
                    text_anchor: "start",
                    dominant_baseline: "middle",
                    pointer_events: "none",
                    "hello"
                }
            })
        })
        .collect();

    rsx! {
        svg {
            view_box: "0 0 400 300",
            width: "100%",
            height: "100%",
            rect {
                x: "30",
                y:"5",
                width:"90%",
                height: "75%",
                stroke: "black",
                stroke_width: "2",
                fill: "#FAF9FF",
                fill_opacity: "92%",
            }
            line {
                x1: "9%",
                y1: "5%",
                x2: "9%",
                y2: "75.3%",
                stroke: "black",
                stroke_width: 4,
            }
            line {
                x1: "9%",
                y1: "75%",
                x2: "95%",
                y2: "75%",
                stroke: "black",
                stroke_width: 4,
            }
            // Tick marks and labels
            { x_tick_elements.into_iter() },
            { y_tick_elements.into_iter() },

            // Scatter points
            { circles.into_iter() },

            text {
            x: "4%",
            y: "45%",
            font_size: "9",
            font_family: "Georgia",
            font_weight: "700",
            text_anchor: "end",
            fill: "white",  // dark gray text for readability on light strip
            "PC1"
            }
            text {
                x: "55%",
                y: "88%",
                font_size: "9",
                font_family: "Georgia",
                font_weight: "700",
                text_anchor: "end",
                fill: "white",  // dark gray text for readability on light strip
                "PC2"
            }
        }
    }
}

#[component]
pub fn CharPlot(mat: MyMmMatrix) -> Element {
    let x_padding = 30;
    let y_padding = 5;
    let pr_comp_data = mat.data_f64;
    /*

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
                    x1: "{x_pos}", y1: "{height - y_padding}", x2: "{x_pos}",
                    y2: "{height - y_padding + 5.0}",  stroke: "black", stroke_width: "1"
                    },
                    text {
                    x: "{x_pos}",  y: "{height - y_padding + 20.0}", font_size: "11",
                    font_family: "Georgia", font_weight: "700", text_anchor: "middle", fill: "black",
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
                        x1: "{x_padding}", y1: "{y_pos}", x2: "{width - x_padding}", y2: "{y_pos}",
                        stroke: "#00bcd4", stroke_width: "0.5", stroke_dasharray: "3 3", stroke_opacity: "0.25"
                    },
                    // Y-axis tick mark
                    line {
                        x1: "{x_padding - 6.0}", y1: "{y_pos}", x2: "{x_padding}",
                        y2: "{y_pos}", stroke: "#90A4AE", stroke_width: "1"
                    },
                    // Label in dark color
                    text {
                        x: "{x_padding - 10.0}", y: "{y_pos + 4.0}", font_size: "11",
                        font_family: "Georgia", font_weight: "700", text_anchor: "end", fill: "#222",
                        "{label}"
                    }
                )
            })
            .collect();
    */
    rsx! {
        svg {
            view_box: "0 0 400 250",
            width:"100%",
            height: "100%",
            rect {
                x: "{x_padding}",
                y: "{y_padding}",
                width:"90%",
                height: "75%",
                stroke: "black",
                stroke_width: "2",
                fill: "#FAF9FF",
                fill_opacity: "92%",
            }
            line {
                x1: "9%",
                y1: "5%",
                x2: "9%",
                y2: "75.3%",
                stroke: "black",
                stroke_width: 4,
            }
            line {
                x1: "9%",
                y1: "75%",
                x2: "95%",
                y2: "75%",
                stroke: "black",
                stroke_width: 4,
            }
            text {
            x: "4%",
            y: "45%",
            font_size: "9",
            font_family: "Georgia",
            font_weight: "700",
            text_anchor: "end",
            fill: "white",  // dark gray text for readability on light strip
            "PC1"
            }
            text {
                x: "55%",
                y: "88%",
                font_size: "9",
                font_family: "Georgia",
                font_weight: "700",
                text_anchor: "end",
                fill: "white",  // dark gray text for readability on light strip
                "PC2"
            }

        }
    }
}
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
            let tick_label = format!("{:.2}", xv); // format as price

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
                    "{tick_label}"
                }
            )
        })
        .collect();

    let y_tick_elements: Vec<_> = y_tick_values
        .iter()
        .map(|&yv| {
            let y_pos = scale_y(yv);
            let tick_label = format!("{}", yv.round() as i64);
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
                    "{tick_label}"
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
