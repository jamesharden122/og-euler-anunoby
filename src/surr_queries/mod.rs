use core::time;

use crate::data_structures::*;
use crate::ops::*;
use nalgebra::DMatrix;
use crate::ops::datetimeops::*;
#[cfg(feature = "server")]
use surrealdb::engine::any;
#[cfg(feature = "server")]
use surrealdb::opt::auth::Root;
#[cfg(feature = "server")]
use surrealdb::{Response, Surreal};
use dioxus::prelude::*;
use chrono::{DateTime, Utc};
#[cfg(feature="server")]
use ml_backend::{
    surreal_queries::{make_db,DbParams},
    polars_ops::{
        select_table_as_df,
        factor_est::*,
        PartEqSurr, Logic,
    }
};
#[cfg(feature = "server")]
use polars::prelude::*;



#[server]
pub async fn query_surr_trademsg_db(
    url: String,
    user: String,
    pass: String,
    ns: String,
    dbname: String,
    time_col: String,
    date1: DateTime<Utc>,
    date2: DateTime<Utc>,
    instrument_id: i64,   // <--- pass the instrument id directly
) -> Result<MyMatrix, ServerFnError> {
    //let db = any::connect("wss://quant-platform-06cb0tpcrpsspao10de28go15s.aws-use1.surreal.cloud").await?;
    let db = make_db(url.as_str(), user.as_str(), pass.as_str(), ns.as_str(), dbname.as_str())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let column_vec = vec!["price", "size", "ts_in_delta", "ts_recv", "bin_1m"];
    println!("{:?}", column_vec.clone());
    let part_eq_surr = PartEqSurr {
        int: Some(vec![("hd.instrument_id".to_string(), instrument_id, Logic::And,)]),
        string: None,//Some(vec![]),
        float: None,
        time_range: match (time_col.clone(), date1, date2) {
            (col, t0, t1) => Some((col, date1.to_rfc3339(), date2.to_rfc3339(), Logic::And)),
            _ => None,
    },
    };
    let mut df = select_table_as_df(&db, "trades", column_vec, part_eq_surr)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let exprs: Vec<Expr> = vec![col("ts_recv")];
    df = df
        .lazy()
        .sort_by_exprs(exprs, SortMultipleOptions::default())
        .collect()
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    df = df.drop(time_col.as_str())
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    println!("dataframe: {:?}", &df);
    // Extract columns
    let mut my_matrix = MyMatrix::from_polars_dataframe(&df)
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    //println!("dataframe: {:?}", &my_matrix.data.clone());
    my_matrix = my_matrix.scale_column(0.000000001, 0).unwrap();
    //my_matrix.data = my_matrix.estimate_retuns().unwrap();
    my_matrix.descrips = my_matrix.snapshot(0).unwrap();
    Ok(my_matrix)
}


#[server]
pub async fn query_surr_trade_bin_db(
    url: String,
    user: String,
    pass: String,
    ns: String,
    dbname: String,
    time_col: String,
    date1: DateTime<Utc>,
    date2: DateTime<Utc>,
    instrument_id: i64,   // <--- pass the instrument id directly
    bin_size: String,
) -> Result<MyMatrix, ServerFnError> {
    //let db = any::connect("wss://quant-platform-06cb0tpcrpsspao10de28go15s.aws-use1.surreal.cloud").await?;
    let db = make_db(url.as_str(), user.as_str(), pass.as_str(), ns.as_str(), dbname.as_str())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let column_vec = vec![ "ret","t0", "t1", "bin",  "mean_price", "p0", "p1","pmax", "pmin", "price_diff"];
    println!("Column Names {:?}", column_vec.clone());
    let part_eq_surr = PartEqSurr {
        int: Some(vec![("instrument_id".to_string(), instrument_id, Logic::And,)]),
        string: Some(vec![("bin_size".to_string(),bin_size, Logic::And)]),
        float: None,
        time_range: match (time_col.clone(), date1, date2) {
            (col, t0, t1) => Some((col, date1.to_rfc3339(), date2.to_rfc3339(), Logic::And)),
            _ => None,
    },
    };
    let mut df = select_table_as_df(&db, "equities_returns", column_vec, part_eq_surr)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let sort_exprs: Vec<Expr> = vec![col(time_col.clone()),];
    let transformation_expr: Vec<Expr> = vec![datetime_to_nanos_expr(col(time_col.clone()))];
    df = df
        .lazy()
        .sort_by_exprs(sort_exprs, SortMultipleOptions::default())
        .apply_exprs(transformation_expr)
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    df = df.drop("t0").map_err(|e| ServerFnError::new(e.to_string()))?;
    df = df.drop("t1").map_err(|e| ServerFnError::new(e.to_string()))?;
    println!("dataframe: {:?}", &df);
    // Extract columns
    let mut my_matrix = MyMatrix::from_polars_dataframe(&df)
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let mp_ind = my_matrix.find_index("mean_price").unwrap();
    let p0_ind = my_matrix.find_index("p0").unwrap();
    let p1_ind = my_matrix.find_index("p1").unwrap();
    let pmin_ind = my_matrix.find_index("pmin").unwrap();
    let pmax_ind = my_matrix.find_index("pmax").unwrap();
    let price_diff_ind = my_matrix.find_index("price_diff").unwrap();

    my_matrix = my_matrix.scale_column(0.000000001, mp_ind).unwrap();
    my_matrix = my_matrix.scale_column(0.000000001, p0_ind).unwrap();
    my_matrix = my_matrix.scale_column(0.000000001, p1_ind).unwrap();
    my_matrix = my_matrix.scale_column(0.000000001, pmax_ind).unwrap();
    my_matrix = my_matrix.scale_column(0.000000001, pmin_ind).unwrap();
    my_matrix = my_matrix.scale_column(0.000000001, price_diff_ind).unwrap();
    //my_matrix.data = my_matrix.estimate_retuns().unwrap();
    my_matrix.descrips = my_matrix.snapshot(0).unwrap();
    println!("dataframe: {:?}", &my_matrix.data.clone());
    Ok(my_matrix)
}

