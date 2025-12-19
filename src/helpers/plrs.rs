#![cfg(feature = "server")]
use polars::prelude::*;

pub fn group_mean(
    mut lf: LazyFrame,
    group_cols: Vec<String>,
    mu_cols: Vec<String>,
) -> PolarsResult<LazyFrame> {
    let expr_gps: Vec<Expr> = group_cols.iter().map(|c| col(c.as_str())).collect();
    let exprs_mu: Vec<Expr> = mu_cols
        .iter()
        .map(|c| col(c.as_str()).mean().alias(format!("{}_{}", c, "mean")))
        .collect();
    lf = lf.group_by(expr_gps).agg(exprs_mu);
    Ok(lf)
}

pub fn group_std(
    mut lf: LazyFrame,
    group_cols: Vec<String>,
    mu_cols: Vec<String>,
) -> PolarsResult<LazyFrame> {
    let expr_gps: Vec<Expr> = group_cols.iter().map(|c| col(c.as_str())).collect();
    let exprs_mu: Vec<Expr> = mu_cols
        .iter()
        .map(|c| col(c.as_str()).std(1).alias(format!("{}_{}", c, "mean")))
        .collect();
    lf = lf.group_by(expr_gps).agg(exprs_mu);
    Ok(lf)
}

pub fn group_mean_dynamic(
    mut lf: LazyFrame,
    group_cols: Vec<String>,
    mu_cols: Vec<String>,
    ind_col: String,
    every: String,
    period: String,
    offset: String,
    label: Label,
) -> PolarsResult<LazyFrame> {
    let dyn_options = DynamicGroupOptions {
        index_column: PlSmallStr::from_string(ind_col.clone()),
        every: Duration::parse(every.as_str()),
        period: Duration::parse(period.as_str()),
        offset: Duration::parse(offset.as_str()),
        label,
        include_boundaries: true,
        closed_window: ClosedWindow::None,
        start_by: StartBy::WindowBound,
    };
    let expr_gps: Vec<Expr> = group_cols.iter().map(|c| col(c.as_str())).collect();
    let exprs_mu: Vec<Expr> = mu_cols
        .iter()
        .map(|c| col(c.as_str()).mean().alias(format!("{}_{}", c, "mean")))
        .collect();
    lf = lf
        .group_by_dynamic(col(ind_col.as_str()), expr_gps, dyn_options)
        .agg(exprs_mu);
    Ok(lf)
}

pub fn group_std_dynamic(
    mut lf: LazyFrame,
    group_cols: Vec<String>,
    mu_cols: Vec<String>,
    ind_col: String,
    every: String,
    period: String,
    offset: String,
    label: Label,
) -> PolarsResult<LazyFrame> {
    let dyn_options = DynamicGroupOptions {
        index_column: PlSmallStr::from_string(ind_col.clone()),
        every: Duration::parse(every.as_str()),
        period: Duration::parse(period.as_str()),
        offset: Duration::parse(offset.as_str()),
        label,
        include_boundaries: true,
        closed_window: ClosedWindow::None,
        start_by: StartBy::WindowBound,
    };
    let expr_gps: Vec<Expr> = group_cols.iter().map(|c| col(c.as_str())).collect();
    let exprs_mu: Vec<Expr> = mu_cols
        .iter()
        .map(|c| col(c.as_str()).std(1).alias(format!("{}_{}", c, "std")))
        .collect();
    lf = lf
        .group_by_dynamic(col(ind_col.as_str()), expr_gps, dyn_options)
        .agg(exprs_mu);
    Ok(lf)
}
