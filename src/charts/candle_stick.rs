use std::{mem::MaybeUninit, ops::Deref};
use crate::ops::MyMatrix;
use crate::Serialize;
use chrono::{NaiveDateTime, TimeZone, Utc};
use dioxus::prelude::*;
use nalgebra::DMatrix;
use serde::Deserialize;
use tracing::info;
#[cfg(feature="server")]
use tokio::runtime::Handle;
#[cfg(feature="server")]
use rayon::prelude::*;


#[derive(Debug, Props, PartialEq, Clone, Serialize, Deserialize)]
pub struct LcMatrix {
    pub matrix: MyMatrix,
    pub y_axis: String,
    pub parallel: bool,
}

#[derive(Debug, Props, PartialEq, Clone, Serialize, Deserialize)]
pub struct Cols { pub t: usize, pub o: usize, pub h: usize, pub l: usize, pub c: usize }
#[derive(Debug, Props, PartialEq, Clone, Copy,Serialize, Deserialize)]
pub struct Candle { pub t: f64, pub o: f64, pub h: f64, pub l: f64, pub c: f64 }
#[derive(Debug, Props, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct PageSpec {pub page: usize, pub page_size: usize}


#[cfg(feature="server")]
pub fn candles_pages(
    mat: &DMatrix<f64>,
    cols: Cols,
    page: PageSpec,
) -> Vec<Candle>
{
    let n = mat.nrows();
    if n == 0 { return vec![]; }

    let start = page.page.saturating_mul(page.page_size);
    if start >= n { return vec![]; }
    let end   = (start + page.page_size).min(n);
    let count = end - start;
    // Keep views alive so slices borrow from a named binding (fixes E0716)
    let tview = mat.column(cols.t);
    let oview = mat.column(cols.o);
    let hview = mat.column(cols.h);
    let lview = mat.column(cols.l);
    let cview = mat.column(cols.c);

    let tcol = tview.as_slice();
    let ocol = oview.as_slice();
    let hcol = hview.as_slice();
    let lcol = lview.as_slice();
    let ccol = cview.as_slice();

      let mut out: Vec<MaybeUninit<Candle>> = {
        let mut v = Vec::with_capacity(count);
        unsafe { v.set_len(count); }
        v
    };

    let handle = Handle::current();
    let chunk_sz = (count / (2 * 4)).clamp(8_192, 65_536).max(1);
    // split the OUTPUT into disjoint &mut chunks => no shared pointer, no Sync issues
    out.as_mut_slice()
        .par_chunks_mut(chunk_sz)
        .enumerate()
        .for_each(|(k, out_chunk)| {
            // absolute index range this chunk covers
            let base_i = start + k * chunk_sz;
            let idx_end = (base_i + out_chunk.len()).min(end);

            // build the absolute indices slice for the async hook
            let idxs: Vec<usize> = (base_i..idx_end).collect();
            // fill this output chunk only
            for (j, cell) in out_chunk.iter_mut().enumerate() {
                let i = base_i + j;
                if i >= end { break; }
                unsafe {
                    cell.as_mut_ptr().write(Candle {
                        t: tcol[i], o: ocol[i], h: hcol[i], l: lcol[i], c: ccol[i],
                    });
                }
            }
        });

    // finalize: transmute MaybeUninit<Candle> -> Candle
    unsafe {
        let ptr = out.as_mut_ptr() as *mut Candle;
        let len = out.len();
        let cap = out.capacity();
        std::mem::forget(out);
        Vec::from_raw_parts(ptr, len, cap)
    }
}

// Server fetcher: NO copies; uses MyMatrix.data (DMatrix) directly
#[server(GetCandlesPage)]
pub async fn get_candles_page(props: LcMatrix, page: PageSpec) -> Result<Vec<Candle>, ServerFnError> {
    // Resolve column indices once (adjust names if yours differ)
    let find = |name: &str| {
        props.matrix.find_index(name)
            .ok_or_else(|| ServerFnError::new(format!("column '{}' not found", name)))
    };
    let cols = Cols {
        t: find("bin")?,     // time
        o: find("p0")?,
        h: find("pmax")?,
        l: find("pmin")?,
        c: find("p1")?,
    };

    let mat: &DMatrix<f64> = &props.matrix.data;
    if mat.nrows() == 0 { return Ok(vec![]); }
    // Use the parallel native builder on server,
    // or a simple direct read on non-server targets to keep builds happy.
    #[cfg(feature="server")]
    {
        let candles = candles_pages(mat, cols, page);
        Ok(candles)
    }

    #[cfg(not(feature="server"))]
    {
        // Fallback: read the page range directly (still zero-copy column access)
        let n = mat.nrows();
        let start = page.page.saturating_mul(page.page_size);
        if start >= n { return Ok(vec![]); }
        let end = (start + page.page_size).min(n);

        let tcol = mat.column(cols.t).as_slice();
        let ocol = mat.column(cols.o).as_slice();
        let hcol = mat.column(cols.h).as_slice();
        let lcol = mat.column(cols.l).as_slice();
        let ccol = mat.column(cols.c).as_slice();

        let mut out = Vec::with_capacity(end - start);
        for i in start..end {
            out.push(Candle { t: tcol[i], o: ocol[i], h: hcol[i], l: lcol[i], c: ccol[i] });
        }
        Ok(out)
    }
}

#[component]
pub fn CandlesChart(props: LcMatrix) -> Element {
    // Chart dims
    let width = 1200.0;
    let height = 1000.0;
    let y_padding = 100.0;
    let x_padding = y_padding / 1.5;
    // stable query (don’t rebuild every render)
    let page = PageSpec { page: 0, page_size: 1000 };

    // cache last successful data so we don’t flicker on route/tab switches
    let last_ok = use_signal(|| Vec::<Candle>::new());

    // kick off server request (don’t `?` / don’t unwrap here)
     let fut = match use_server_future({
        let props = props.clone();
        move || get_candles_page( LcMatrix {
                matrix: props.matrix.clone(),
                y_axis: props.y_axis.clone(),
                parallel: props.parallel,
            }, page)
    }) {
        Ok(r) => r,
        Err(e) => {
            return rsx!(svg {
            width: "{width}",
            height: "{height}",
            style: "border: background-color: #0a0f0a;",

            // plot area background
            rect {
                x: "{x_padding}",
                y: "{y_padding / 4.0}",
                width: "{width - x_padding * 2.0}",
                height: "{height - y_padding - y_padding / 4.0}",
                fill: "rgba(0, 128, 0, 0.05)",
                }
            })
        }
    };

    // update cache when the future resolves (no writes during render)
    use_effect({
        let mut last_ok = last_ok.clone();
        move || {
            if let Some(Ok(candles)) = fut.read().deref() {
                last_ok.set(candles.clone());
            }
            }
    });

    // pick data to render: prefer cache; else use fresh; else skeleton
    let candles: Vec<Candle> = if !last_ok.read().is_empty() {
        last_ok.read().clone()
    } else if let Some(Ok(c)) = fut.read().deref() {
        c.clone()
    } else {
        return rsx!(svg {
            width: "{width}",
            height: "{height}",
            style: "border: background-color: #0a0f0a;",

            // plot area background
            rect {
                x: "{x_padding}",
                y: "{y_padding / 4.0}",
                width: "{width - x_padding * 2.0}",
                height: "{height - y_padding - y_padding / 4.0}",
                fill: "rgba(0, 128, 0, 0.05)",
                }
            })
    };

    // guard: empty data
    if candles.is_empty() {
        return rsx!(svg {
            width: "{width}",
            height: "{height}",
            style: "border: background-color: #0a0f0a;",

            // plot area background
            rect {
                x: "{x_padding}",
                y: "{y_padding / 4.0}",
                width: "{width - x_padding * 2.0}",
                height: "{height - y_padding - y_padding / 4.0}",
                fill: "rgba(0, 128, 0, 0.05)",
                }
            })
    }



    // Bounds
    let x_min = candles.iter().map(|c| c.t).fold(f64::INFINITY, f64::min);
    let x_max = candles.iter().map(|c| c.t).fold(f64::NEG_INFINITY, f64::max);
    let y_min = candles.iter().map(|c| c.l).fold(f64::INFINITY, f64::min);
    let y_max = candles.iter().map(|c| c.h).fold(f64::NEG_INFINITY, f64::max);
    let x_span = if x_max > x_min { x_max - x_min } else { 1.0 };
    let y_span = if y_max > y_min { y_max - y_min } else { 1.0 };

    let inner_w = width - 2.0 * x_padding;
    let inner_h = height - 2.0 * y_padding;
    let scale_x = |x: f64| x_padding + (x - x_min) / x_span * inner_w;
    let scale_y = |y: f64| (y_max - y) / y_span * inner_h + y_padding;

    // --- ticks (place after scale_x/scale_y) ---
let xticks = 15usize;
let yticks = 15usize;

// pick evenly spaced numeric values across the domains
let x_tick_values: Vec<f64> = (0..xticks)
    .map(|i| x_min + (i as f64 / (xticks.saturating_sub(1).max(1)) as f64) * (x_max - x_min))
    .collect();

let y_tick_values: Vec<f64> = (0..yticks)
    .map(|i| y_min + (i as f64 / (yticks.saturating_sub(1).max(1)) as f64) * (y_max - y_min))
    .collect();

// build RSX nodes for ticks/labels
let x_tick_elements: Vec<_> = x_tick_values
    .iter()
    .map(|&xv| {
        let x_pos = scale_x(xv);

        // interpret xv as ns since epoch; be careful with casts
        let ns = xv.round() as i64;
        let timestamp = chrono::Utc.timestamp_nanos(ns);
        let formatted = timestamp.format("%Y-%m-%d %H:%M:%S").to_string();

        rsx!(
            // small tick on axis
            line {
                x1: "{x_pos}",
                y1: "{height - y_padding}",
                x2: "{x_pos}",
                y2: "{height - y_padding + 5.0}",
                stroke: "black",
                stroke_width: "1"
            }
            // rotated label
            text {
                x: "{x_pos}",
                y: "{height - y_padding + 20.0}",
                font_size: "11",
                font_family: "Georgia",
                font_weight: "700",
                text_anchor: "start",
                fill: "black",
                transform: "rotate(40, {x_pos}, {height - y_padding})",
                "{formatted}"
            }
        )
    })
    .collect();

let y_tick_elements: Vec<_> = y_tick_values
    .iter()
    .map(|&yv| {
        let y_pos = scale_y(yv);
        let label = format!("{:.2}", yv);

        rsx! {
            // light horizontal grid line
            line {
                x1: "{x_padding}",
                y1: "{y_pos}",
                x2: "{width - x_padding}",
                y2: "{y_pos}",
                stroke: "#ccc",
                stroke_width: "0.5",
                stroke_dasharray: "4 2",
                stroke_opacity: "0.6"
            }
            // small tick on y-axis
            line {
                x1: "{x_padding - 5.0}",
                y1: "{y_pos}",
                x2: "{x_padding}",
                y2: "{y_pos}",
                stroke: "black",
                stroke_width: "1"
            }
            // numeric label
            text {
                x: "{x_padding - 8.0}",
                y: "{y_pos + 4.0}",
                font_size: "11",
                font_family: "Georgia",
                font_weight: "700",
                text_anchor: "end",
                fill: "black",
                "{label}"
            }
        }
    })
    .collect();


    // Candle body width from median spacing
    let mut xs: Vec<f64> = candles.iter().map(|c| c.t).collect();
    xs.sort_by(|a,b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut gaps: Vec<f64> = xs.windows(2).map(|w| w[1]-w[0]).filter(|g| *g>0.0).collect();
    gaps.sort_by(|a,b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_gap = if gaps.is_empty() { 1.0 } else { gaps[gaps.len()/2] };
    let px_gap = (median_gap / x_span) * inner_w;
    let body_w = (px_gap * 0.6).clamp(1.0, 30.0);

    rsx! {
        svg {
            width: "{width}",
            height: "{height}",
            style: "border: background-color: #0a0f0a;",

            // plot area background
            rect {
                x: "{x_padding}",
                y: "{y_padding / 4.0}",
                width: "{width - x_padding * 2.0}",
                height: "{height - y_padding - y_padding / 4.0}",
                fill: "rgba(0, 128, 0, 0.05)",
            }

            // axes
            line { x1: "{x_padding}", y1: "{height - y_padding}", x2: "{width - x_padding}", y2: "{height - y_padding}", stroke: "#81c784", stroke_width: "1" }
            line { x1: "{x_padding}", y1: "{y_padding}", x2: "{x_padding}", y2: "{height - y_padding}", stroke: "#81c784", stroke_width: "1" }

            // candles (no allocations in loop beyond rsx nodes)
            {
                candles.iter().map(|c| {
                    let x   = scale_x(c.t);
                    let y_o = scale_y(c.o);
                    let y_c = scale_y(c.c);
                    let y_h = scale_y(c.h);
                    let y_l = scale_y(c.l);

                    let up = c.c >= c.o;
                    let fill   = if up { "#66bb6a" } else { "#ef5350" };
                    let stroke = if up { "#43a047" } else { "#e53935" };

                    let bx = x - body_w/2.0;
                    let by = y_o.min(y_c);
                    let bh = (y_o - y_c).abs().max(1.0);

                    rsx! {
                        // wick
                        line { x1: "{x}", y1: "{y_h}", x2: "{x}", y2: "{y_l}", stroke: "{stroke}", stroke_width: "1.2" }
                        // body
                        rect { x: "{bx}", y: "{by}", width: "{body_w}", height: "{bh}", 
                        fill: "{fill}", stroke: "{stroke}", stroke_width: "0.8", rx: "1.5", ry: "1.5" }
                                    // X and Y ticks and labels
                    }
                })
            }
            { x_tick_elements.clone().into_iter() },
            { y_tick_elements.clone().into_iter() },
            text {
                        x: "20.0",
                        y: "{height / 2.0}",
                        transform: "rotate(-90, 20.0, {height / 2.0})",
                        font_size: "14",
                        font_family: "Georgia",
                        font_weight: "700",
                        text_anchor: "middle",
                        fill: "black",
                        "Price"
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
                        "Timestamp"
                    }
        }
    }
}
