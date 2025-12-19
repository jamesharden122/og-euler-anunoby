use crate::helpers::{self, dropdownlist};
#[cfg(feature = "server")]
use crate::{
    dbinst::{duckstore, SharedDuck},
    helpers::plrs,
};
use crate::{
    news::fetch,
    ops::multi_type_mat::MyMmMatrix,
    prompting::PromptBox,
    tables::{MultiFactorDisplay, SecurityComp},
};
use chrono::{NaiveDate, TimeZone, Utc};
use dioxus::prelude::*;
#[cfg(feature = "server")]
use polars::prelude::*;
use std::env::temp_dir;
use std::ops::Deref;
// tokio is not needed directly here

//Multi-asset data tables
//Instantiate duck db start_duck_db(max_mem: &str, thread_count: i64,) -> Result<Connection, crate::error::AppError>
//Set DB Type based on country and asset type  dbtype = DbType::GlobalDailyIndex;
//Create features if not pre-computed
//Create filters for the top factors table on date, factors, bins, Securiti

#[cfg(feature = "server")]
mod dbfac {
    use super::*;
    use dioxus::prelude::ServerFnError;
    use duckdb::Connection;
    use polars::prelude::*;
    use std::sync::{Arc, Mutex};
    use wrds_io;

    /// Start an in-memory DuckDB instance via the words_db helper and return the connection.
    /// Later, we can promote this to a global `OnceCell<Arc<Connection>>` if needed.

    pub async fn db_inst(factors_path: String) -> Result<SharedDuck, ServerFnError> {
        let conn = wrds_io::instantiatedb::duckdbinst::start_duck_db("8GB", 14)
            .await
            .map_err(|e| ServerFnError::new(format!("duckdb start error: {:?}", e)))?;
        let arc_conn = Arc::new(Mutex::new(conn.try_clone().unwrap()));
        wrds_io::instantiatedb::duckdbinst::DbType::EquityFactorsMonthly
            .ingest(arc_conn.clone(), &factors_path)
            .await
            .map_err(|e| {
                ServerFnError::new(format!("ingest equity factors monthly parquet: {:?}", e))
            })?;
        Ok(arc_conn)
    }
    pub async fn query_factors_range(
        conn: Arc<Mutex<Connection>>,
        country: &str,
        factors: Vec<String>,
        start_iso: NaiveDate,
        end_iso: NaiveDate,
    ) -> PolarsResult<DataFrame> {
        let df_rows =
            wrds_io::finance_data_structs::equity_factors::EquityFactorsMonthly::read_range(
                conn,
                (start_iso, end_iso),
            )
            .await
            .map_err(|e| PolarsError::ComputeError(format!("read_range: {e:?}").into()))?;
        tracing::debug!("set data rows");
        let mut fac_df = <wrds_io::finance_data_structs::equity_factors::EquityFactorsMonthly as wrds_io::finance_data_structs::ToPolars>::df_from_rows(&df_rows)
            .map_err(|e| PolarsError::ComputeError(format!("df_from_rows: {e:?}").into()))?;
        tracing::debug!("Initiate DataFrame");
        let factors_expr: Vec<Expr> = factors.iter().map(|c| col(c.as_str())).collect();
        tracing::debug!("shape: {:?}", &fac_df.shape());
        fac_df = fac_df
            .lazy()
            .select(factors_expr)
            .filter(col("excntry").eq(lit(country)))
            .sort_by_exprs(
                vec![col("excntry"), col("gvkey"), col("date")],
                Default::default(),
            )
            .collect()
            .map_err(|e| PolarsError::ComputeError(format!("collect: {e:?}").into()))?;
        tracing::debug!("shape: {:?}", &fac_df.shape());

        Ok(fac_df)
    }
}

// Return a serializable matrix; keep connection on server side
#[server]
pub async fn fetch_factors_matrix(
    country: String,
    factors: Vec<String>,
    start_iso: NaiveDate,
    end_iso: NaiveDate,
    factors_path: String,
    gby: Option<Vec<String>>,
) -> Result<(MyMmMatrix, MyMmMatrix), ServerFnError> {
    #[cfg(not(feature = "server"))]
    {
        let _ = (country, factors, start_iso, end_iso, factors_path, gby);
        return Err(ServerFnError::new(
            "fetch_factors_matrix requires the `server` feature",
        ));
    }

    #[cfg(feature = "server")]
    {
        tracing::debug!("Retrieving Factors");
        if duckstore::get().await.is_none() {
            tracing::debug!("build data from parquet file");
            _ = duckstore::set(dbfac::db_inst(factors_path).await?).await;
        } else {
            tracing::debug!("Data already loaded into memory no need to rebuild from parquet");
        }
        tracing::debug!("Factors sub-sample");
        let mut df = dbfac::query_factors_range(
            duckstore::get().await.unwrap(),
            country.as_str(),
            factors,
            start_iso,
            end_iso,
        )
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
        let mat1 = MyMmMatrix::from_polars_dataframe(&mut df)
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        tracing::debug!("data float shape{:?}", mat1.colnames_enum_f64);
        tracing::debug!("data str shape{:?}", mat1.colnames_enum_str);
        let mut mat2 = MyMmMatrix::new(0, 0);
        if let Some(group_cols) = gby {
            let schm = df.schema().clone();
            let mut nms: Vec<String> = Vec::new();
            for tup in schm.iter() {
                if group_cols.iter().any(|c| c == tup.0.as_str()) {
                    continue;
                }
                match tup.1 {
                    DataType::Float64
                    | DataType::Float32
                    | DataType::Int64
                    | DataType::Int32
                    | DataType::Int16
                    | DataType::Int8
                    | DataType::UInt64
                    | DataType::UInt32
                    | DataType::UInt16
                    | DataType::UInt8 => nms.push(tup.0.to_string()),
                    _ => {}
                }
            }
            df = plrs::group_mean(df.lazy(), group_cols, nms)
                .map_err(|e| ServerFnError::new(e.to_string()))?
                .collect()
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            mat2 = MyMmMatrix::from_polars_dataframe(&mut df)
                .map_err(|e| ServerFnError::new(e.to_string()))?;
        }
        Ok((mat1, mat2))
    }
}

#[component]
pub fn MultiAsset() -> Element {
    let mut selected = use_signal(|| "TUR".to_string());
    let mut factor = use_signal(|| "Price".to_string());
    let mut factor_list = use_signal(|| {
        vec![
            String::from("date"),
            String::from("gvkey"),
            String::from("iid"),
            String::from("excntry"),
            String::from("dolvol"),
            String::from("rvolhl_21d"),
            String::from("ret_60_12"),
            String::from("ret_3_1"),
        ]
    });
    let mut idcols: Signal<Vec<String>> = use_signal(|| Vec::new());
    let mut idiofac: Signal<Vec<String>> = use_signal(|| Vec::new());
    let mut crossfac: Signal<Vec<String>> = use_signal(|| Vec::new());
    let mut bin_size = use_signal(|| "5m".to_string());
    let mut start_date = use_signal(|| Utc::now().to_rfc3339());
    let mut end_date = use_signal(|| Utc::now().to_rfc3339());
    let start_naivedate = use_signal(|| Utc.with_ymd_and_hms(2024, 1, 1, 0, 1, 1).unwrap());
    let end_naivedate = use_signal(|| Utc::now());
    // No client-side connection cache; DB lives server-side only
    // Build a comma-separated list efficiently (no leading comma) when needed
    // (displayed below the Factors dropdown)
    let mut mat_fac: Signal<Option<MyMmMatrix>> = use_signal(|| None);
    let mut gp_mat_fac: Signal<Option<MyMmMatrix>> = use_signal(|| None);

    let factors_path =
        "~/Dropbox/Desktop/tesero-sol/software_development/trading/data/raw_files/parquet/factors/global/monthly/2020_2025_parquet.parquet"
            .to_string();

    // Run the server fetch in the background; use_resource won't suspend initial render
    let resource = use_resource({
        move || {
            let country = selected();
            let factors = factor_list();
            let start = start_naivedate().date_naive();
            let end = end_naivedate().date_naive();
            let path = factors_path.clone();
            async move {
                fetch_factors_matrix(
                    country,
                    factors,
                    start,
                    end,
                    path,
                    Some(vec![
                        String::from("gvkey"),
                        String::from("iid"),
                        String::from("excntry"),
                    ]),
                )
                .await
            }
        }
    });
    use_effect({
        let resource = resource.clone();
        move || {
            if let Some(Ok(mat)) = resource.read().deref() {
                mat_fac.set(Some(mat.0.clone()));
                gp_mat_fac.set(Some(mat.1.clone()));
                tracing::debug!("Succesfully update the matrix");
            }
        }
    });
    rsx! {
        div { class: "card",
            div {
                label { "Countries" }
                select {
                    value: "{selected()}",
                    onchange: move |evt| selected.set(evt.value()),
                    option { value: "TUR", "TUR" }
                    option { value: "CHN", "CHN" }
                    option { value: "HKG", "HKG" }
                    option { value: "DNK", "DNK" }
                    option { value: "PRT", "PRT" }
                    option { value: "COL", "COL" }
                    option { value: "IDN", "IDN" }
                    option { value: "CHL", "CHL" }
                    option { value: "HUN", "HUN" }
                    option { value: "JPN", "JPN" }
                    option { value: "GBR", "GBR" }
                    option { value: "ITA", "ITA" }
                    option { value: "NLD", "NLD" }
                    option { value: "SWE", "SWE" }
                    option { value: "AUT", "AUT" }
                    option { value: "FRA", "FRA" }
                    option { value: "NZL", "NZL" }
                    option { value: "KOR", "KOR" }
                    option { value: "DEU", "DEU" }
                    option { value: "AUS", "AUS" }
               }
                label { "Factors"}
                dropdownlist::SelectOptions {
                    value: factor(),
                    options: dropdownlist::SelectOptionProps::global_stock_returns(),
                    onchange: move |evt: FormEvent| {
                        let val = evt.value();
                        factor.set(val.clone());
                        factor_list.with_mut(|v| v.push(val));
                    },
                }                // Show all selected factors below the dropdown
                button {
                    class: "ma-btn",
                    onclick: move |_| factor_list.set(Vec::<String>::new()),
                    "Clear Factors"
                }
                div { class: "selected-factors",
                    label { "Selected" }
                    p { "{factor_list().join(\", \")}" }
                }
            }
            div {
                label { "Bin Size" }
                select {
                    value: "{bin_size()}",
                    onchange: move |evt| bin_size.set(evt.value()),
                    option { value: "1m", "1 minute" }
                    option { value: "5m", "5 minutes" }
                    option { value: "30m", "30 minutes" }
                    option { value: "1hour", "1 hour" }
                    option { value: "1day", "1 day" }
                }
                label { "Start Date" }
                input {
                    r#type: "datetime-local",
                    value: "{start_date().as_str()}",
                    oninput: move |e| start_date.set(e.value()),
                }
                label { "End Date" }
                input {
                    r#type: "datetime-local",
                    value: "{end_date().as_str()}",
                    oninput: move |e| end_date.set(e.value()),
                }
            }
        }
        div {h2 {}}
        {

            match gp_mat_fac() {
                Some(mat) => rsx!(
                    MultiFactorDisplay {
                        mat,
                        cross_factors: Some(vec!["dolvol_mean".to_string()]),
                        idiosyn_factors: Some(vec![
                            "rvolhl_21d".to_string(),
                            String::from("rvolhl_21d_mean"),
                            String::from("ret_60_12_mean"),
                            String::from("ret_3_1_mean"),
                        ]),
                        id_cols: vec![
                            "gvkey".to_string(),
                            "iid".to_string(),
                            "excntry".to_string(),
                        ],
                    }
                ),
                None => match resource.read().deref() {
                    Some(Err(e)) => rsx!(div { class: "error", "Factor load failed: {e}" }),
                    _ => rsx!(div { "Loading factors…" }),
                }
            }
        }
        section { class: "grid-wrapper",
                PromptBox { }
            }
        section { class: "grid-wrapper",
            div{
                style: "width: 55%",
                div {
                    label { "Visuals" }
                    select {
                        value: "{bin_size()}",
                        option { value: "PCA", "PCA" }
                    }
                }
            }
            div {
                div {
                    label { "Security" }
                    input {
                        r#type: "text",
                        value: "",
                    }
                    button {
                        class: "ma-btn",
                        "k-means neighbor"
                    }
                    button {
                        class: "ma-btn",
                        "Similar Corporate"
                    }
                }
                div {
                    h3 {"Group Snapshot"}
                    SecurityComp {}
                }
            }
        }
        match mat_fac() {
                Some(mat) => rsx!(
                    MultiFactorDisplay {
                        mat,
                        cross_factors: Some(vec!["dolvol".to_string()]),
                        idiosyn_factors: Some(vec![
                            "rvolhl_21d".to_string(),
                            String::from("rvolhl_21d"),
                            String::from("ret_60_12"),
                            String::from("ret_3_1"),
                        ]),
                        id_cols: vec![
                            "date".to_string(),
                            "gvkey".to_string(),
                            "iid".to_string(),
                            "excntry".to_string(),
                        ],
                    }
                ),
                None => match resource.read().deref() {
                    Some(Err(e)) => rsx!(div { class: "error", "Factor load failed: {e}" }),
                    _ => rsx!(div { "Loading factors…" }),
                }
            }
    }
}
