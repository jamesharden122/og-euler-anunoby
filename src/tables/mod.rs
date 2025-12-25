use crate::ops::{multi_type_mat::MyMmMatrix, MyMatrix};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[component]
pub fn SalesTable(props: MyMatrix) -> Element {
    rsx! {
        table { class: "table_cls",
            thead {
                tr {
                    th { class: "text-left-header", "Descriptive Statistics Table"}
                    th { class: "text-right-header", }
                }
            }
            tbody { class: "table-hover",
                tr {
                    td { class: "text-left", "Mean"}
                    td { class: "text-right", "{props.descrips.0}"}
                }
                tr {
                    td { class: "text-left", "Volatility(SD)"}
                    td { class: "text-right", "{props.descrips.1}"}
                }
                tr {
                    td { class: "text-left",  "Volume"}
                    td { class: "text-right",  "{props.descrips.2}"}
                }
                tr {
                    td { class: "text-left", "Grouping" }
                    td { class: "text-right", "{props.descrips.3}"}
                }
                tr {
                    td { class: "text-left", "Stoch Vol." }
                    td { class: "text-right", "{props.descrips.4}"}
                }
                tr {
                    td {class: "text-left", "Buy Score"}
                    td {class: "text-right","{props.descrips.5}"}
                }
            }
        }
    }
}

//#[component]
pub fn TradeDisplay(trades: MyMatrix) -> Element {
    let nrow = trades.data.nrows() - 1;
    println!("row number{:?}", nrow);
    rsx! {
        table {
            class: "trade-table",
            thead {
                tr {
                    th { "Price" }
                    th { "Size" }
                    th { "Time-Stamp" }
                }
            }
            tbody {
                for (_i,trade) in trades.data.view((0,0),(nrow,4)).row_iter().enumerate() {
                    tr {class: "ind-trade",
                            td { "{trade[0]}" } // Price
                            td { "{trade[1]}" } // Size
                            td { "{MyMatrix::convert_nano_to_datetime(trade[3]-trade[2]).unwrap()}" } // TS Recv
                    }
                }
            }
        }
    }
}

#[derive(Debug, Props, PartialEq, Clone, Serialize, Deserialize)]
pub struct MyMmMatrixandFacs {
    pub mat: MyMmMatrix,
    pub cross_factors: Option<Vec<String>>,
    pub idiosyn_factors: Option<Vec<String>>,
    pub id_cols: Vec<String>,
}
//In the prop need:
//      - cross-sectional factor count
//      - idiosyncratic factor count
#[component]
pub fn MultiFactorDisplay(props: MyMmMatrixandFacs) -> Element {
    let nrow = props.mat.data_f64.nrows();
    let cf = props.cross_factors.unwrap();
    let idif = props.idiosyn_factors.unwrap();
    let idc = props.id_cols;
    let str_data = props.mat.data_str;
    let float_data = props.mat.data_f64;
    let str_cols = props.mat.colnames_enum_str.unwrap();
    let float_cols = props.mat.colnames_enum_f64.unwrap();
    let mut count = use_signal(|| 0);
    let render_cell = |row: usize, column_name: &String| -> String {
        if let Some((j, _)) = str_cols.iter().find(|(_, s)| s == column_name) {
            str_data
                .get((row, *j))
                .map(|v| v.to_string())
                .unwrap_or_default()
        } else if let Some((j, _)) = float_cols.iter().find(|(_, s)| s == column_name) {
            float_data
                .get((row, *j))
                .map(|v| v.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        }
    };
    fn render_cell_6(s: String) -> String {
        if let Ok(x) = s.parse::<f64>() {
            format!("{:.6}", x)
        } else {
            s
        }
    }

    rsx! {
        div {class: "trade-table-wrap",
        table {class: "trade-table-mf",
            thead {
                tr {
                    th{colspan: idc.len(), ""}
                    th {colspan: cf.len(),"Cross-Sectional Factors"}
                    th {colspan: idif.len(),"Idiosyncratic Factors"}
                }
                tr {
                    for nm in idc.iter() {
                        th { id: "secid", "{nm}" }
                    }
                    for nm in cf.iter() {
                        th { id: "fact", "{nm}" }
                    }
                    for nm in idif.iter() {
                        th { id: "fact", "{nm}"}
                    }
                }
            }
            tbody {
                for i in 1..nrow {
                    tr {class: "ind-trade",
                        for nm in idc.iter() { td { id: "secid", "{render_cell(i, nm)}" } }
                        for nm in cf.iter() { td { id: "fact", "{render_cell_6(render_cell(i, nm))}" } }
                        for nm in idif.iter() { td { id: "fact", "{render_cell_6(render_cell(i, nm))}" } }
                    }
                }
            }
        }
        }
    }
}

pub fn SecurityComp() -> Element {
    rsx! {
        table { class: "kv-table",
            tbody {
            tr { th{"AAPL"} td{"hi"} td{"hi"} td{"hi"} }
            tr { th{"TSLA"} td{""} td{""} td{""} }
            tr { th{"IBM"}  td{""} td{""} td{""} }
            tr { th{"MSFT"} td{""} td{""} td{""} }
            tr { th{"CNVA"} td{""} td{""} td{""} }
            }
        }
    }
}
