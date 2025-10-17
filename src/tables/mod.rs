use crate::ops::MyMatrix;
use dioxus::prelude::*;

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
