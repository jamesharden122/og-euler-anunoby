use serde::Deserialize;
use crate::{
    charts::{
        clustering::ScatterPlot, single_asset_lc::PlottersChart,
        candle_stick::{CandlesChart,LcMatrix},
        ChartType,
    },
    ops::MyMatrix,
    tables::{SalesTable, TradeDisplay},
    prompting::PromptBox,
};
use crate::model_request::{*,momentum_lstm::*};
use crate::surr_queries::{query_surr_trademsg_db,query_surr_trade_bin_db};
use crate::news::Fetch;
use dioxus::prelude::*;
use chrono::{DateTime,Utc};
//use ml_backend::surreal_queries::{make_db,DbParams};

#[derive(Default, Clone, PartialEq, Debug, Deserialize)]
struct BacktestResult {
    pub sharpe: f64,
    pub sortino: f64,
    pub mdd: f64,
    pub t_stat: f64,
    pub information_ratio: Option<f64>, // because it can be null
    pub rows: usize,
    pub cols: usize,
    pub path: String,
}


#[derive(Clone, PartialEq, Debug, Deserialize, Default)]
struct PhaseStats {
    mu_daily: f64,
    sigma_daily: f64,
    sharpe_ann: f64,
    avg_position: f64,
    avg_turnover: f64,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Default)]
struct SplitStats {
    #[serde(default)] train: PhaseStats,
    #[serde(default)] val:   PhaseStats,
    #[serde(default)] test:  PhaseStats,
}


#[component]
pub fn SingleAsset() -> Element {
	let mut response = use_signal(MyMatrix::new10x);
    let mut response2 = use_signal(MyMatrix::new10x);
    // initialize signals with a DateTime<Utc>
    let mut date1 = use_signal(Utc::now);
    let mut date2 = use_signal(Utc::now);
    let mut start_date = use_signal(||Utc::now().to_rfc3339());
    let mut end_date = use_signal(||Utc::now().to_rfc3339());
    let mut instrument = use_signal(|| { 8147});
    let mut bin_size = use_signal(|| "5m".to_string());
    let mut chart_type = use_signal(|| 0);
    let url = use_signal(|| { String::from("https://quant-platform-06cb0tpcrpsspao10de28go15s.aws-use1.surreal.cloud/rpc")});
    let user = use_signal(|| {String::from("root")});
    let pass = use_signal(|| {String::from("root")});
    let ns = use_signal(|| {String::from("equities")});
    let db = use_signal(|| {String::from("historical")});
    let mut result_body_bt = use_signal(|| {BacktestResult::default()});
    let mut result_body_trn = use_signal(|| {SplitStats::default()});
    //========================================================
    //Model Request Params
    //========================================================
    let mut train_mom = use_signal(|| TrainSpec {
        trainer_path: "../ml-project/models/mls_lstm_trainer.py".into(),
        trainer_class: "MLSLSTMTrainer".into(),
        time_steps: 5, input_dim: 6,
        val_split: 0.1, test_split: 0.1, epochs: 10,
        batch_size: None, verbose: 1, shuffle_before_split: false,
        seed: 42, save_every_epoch: false, save_weights_only: false,
        monitor: "val_loss".into(), save_best_only: true,
        feature_names: serde_json::json!([{"MomFactor": null}]),
        feature_spec: serde_json::json!({
            "mean_price":"float32", "ret_sma20":"float32", "ret_sma50":"float32",
            "ret_ema_small_pt1":"float32","ret_ema_large_pt6":"float32","ret_macd1s6l":"float32"
        }),
        tfrecord_out: "../tmp_data/mom_data_sharpe.tfrecord".into(),
        writer_path: "../ml-project/py/pl2tfrecord_writer.py".into(),
        reader_path: "../ml-project/py/pl2tfrecord_reader.py".into(),
        sequence_length: 5, horizon: 1, stride: 1,
        group_col: "instrument_id".into(),
        return_col: "ret".into(), sigma_col: "sigma".into(), cost_col: "cost".into(),
        include_cost: true, gzip: true, shuffle: false,
    });
    let mut bt_mom = use_signal(|| BacktestSpec {
        feats: "mom".into(),
        model_path: "../ml-project/models/saved/test/final_model.onnx".into(),
        out_csv: "../tmp_data/my_bt.csv".into(),
    });
    let get_data_button = {
        let log_in = move |_| {
            let date_fmt = "%Y-%m-%dT%H:%M:%S%z";
            date1.set(DateTime::parse_from_str((start_date().clone() + ":00+0000").as_str(), date_fmt).unwrap().into());
            println!("{:?}", date1);    
            date2.set(DateTime::parse_from_str((end_date().clone() + ":00+0000").as_str(), date_fmt).unwrap().into());  
            println!("{:?}", date2);          
            spawn(async move {
                if let Ok(resp) = query_surr_trademsg_db(
                    url(),
                    user(),
                    pass(),
                    ns(),
                    db(),
                    "bin_1m".to_string(),
                    date1(),
                    date2(),
                    instrument(),
                )
                .await
                {
                    response.set(resp);
                }
            });
            spawn(async move {

                if let Ok(resp) = query_surr_trade_bin_db(
                    url(),
                    user(),
                    pass(),
                    ns(),
                    db(),
                    "bin".to_string(),
                    date1(),
                    date2(),
                    instrument(),
                    bin_size.to_string(),
                )
                .await
                {
                    response2.set(resp);
                }
            });
        };
        Some(rsx!(button { onclick: log_in, "Get Data" }))
    };
	let data = response.read();
    let descrips = data.snapshot(0).unwrap_or_default();
    let data2 = response2.read();
    let descrips2 = data2.snapshot(1).unwrap_or_default();
    rsx! {
        div { class: "card",
            div {
                label {"Asset Search"}
                    input {
                        r#type: "number",       // HTML input of type "number"
                    value: "{instrument()}",
                    oninput: move |evt| {
                        if let Ok(val) = evt.value().parse::<i64>() {
                            instrument.set(val);
                        }
                    }
                }
                label {"Bin Size"}
                    select {
                    value: "{bin_size()}",
                    onchange: move |evt| {
                        bin_size.set(evt.value());
                    },
                    option { value: "1m", "1 minute" }
                    option { value: "5m", "5 minutes" }
                    option { value: "30m", "30 minutes" }
                    option { value: "1hour", "1 hour" }
                    option { value: "1day", "1 day" }
                }
            }
            div {
                label { "Start Date" }
                input {
                    r#type: "datetime-local",
                    value: "{start_date().as_str()}",
                    oninput: move |e| {
                        if let Ok(temp_val) = e.value().parse::<String>(){
                            start_date.set(temp_val)
                        }
                    }
                }
                label { "End Date" }
                input {
                    r#type: "datetime-local",
                    value: "{end_date.read().as_str()}",
                    oninput: move |e| {
                        if let Ok(temp_val) = e.value().parse::<String>(){
                            end_date.set(temp_val)
                        }
                    }
                }
            }
            div {
            }
            div {
                label { "Chart Type" }
                select {
                    value: "{ChartType::from_usize_as_str(chart_type())}",
                    onchange: move |evt| {
                        chart_type.set(evt.value().parse::<usize>().unwrap_or(0));
                    },
                    option { value: 0, "Line chart" }
                    option { value: 1, "Candle Stick" }
                }
            }
        }
        { get_data_button }
        section { class: "grid-wrapper",
            section { class: "grid-section-1x",
                section { class: "grid-section-2x",
                    div { class: "grid-item",
                        SalesTable { data: data.data.clone(), descrips: descrips }
                    }
                    div { class: "grid-item",
                        ScatterPlot { data: data.data.clone(), descrips: descrips }
                    }
                }
                section { class: "grid-section-1x",
                    div { class: "grid-item",
                        Fetch {}
                    }
                }
            }
            section { class: "grid-section-1x",
                div { class: "grid-full-chart",
                    match chart_type() {
                        0 => rsx! {
                                PlottersChart {
                                    matrix: data2.clone(),
                                    y_axis: "mean_price".to_string(),
                                }
                            },
                        1 => rsx! {
                                CandlesChart {
                                    matrix: data2.clone(),
                                    y_axis: "mean_price".to_string(),
                                    parallel: true
                            }
                        },
                        _ => rsx! {
                            "None"
                        }
                    }
                }    
            }
        }
        section { class: "grid-wrapper",
                PromptBox { }
         }

        
        UiProvider {  
            section { class: "backtest-table-container",
                div { class: "backtest-choice-params",
                    div { class: "form-section-wrapper",
                        h3 { class:"qh3-term", "Parameters" }
                        div { class: "form-section",
                             ParamsForm {  }
                         }
                     }
                    div { class: "button-section-wrapper",
                        h3 { class:"qh3-term", "RNN Factors" }
                        div { class: "button-section",
                             ModelBlock {
                                name: "DNN-8-8-8",
                                description: "Fully connected deep network",
                                parameters: "1.2M",
                                action: vec![(String::from("backtest"), ModelAction::MomBacktest(bt_mom())),(String::from("train"), ModelAction::MomTrain(train_mom()))],
                                on_result_bt: move |body: String| result_body_bt.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                                on_result_trn: move |body: String| result_body_trn.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                            }
                            ModelBlock { 
                                name: "CNN-16-32",
                                description: "Convolutional neural net",
                                parameters: "4.5M",
                                action: vec![(String::from("backtest"), ModelAction::MomBacktest(bt_mom())),(String::from("train"), ModelAction::MomTrain(train_mom()))],
                                on_result_bt: move |body: String| result_body_bt.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                                on_result_trn: move |body: String| result_body_trn.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                            }
                            ModelBlock { 
                                name: "LSTM-64",
                                description: "Long short-term memory net",
                                parameters: "850K",
                                action: vec![(String::from("backtest"), ModelAction::MomBacktest(bt_mom())),(String::from("train"), ModelAction::MomTrain(train_mom()))],
                                on_result_bt: move |body: String| result_body_bt.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                                on_result_trn: move |body: String| result_body_trn.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                            }
                            ModelBlock { 
                                name: "Transformer-Base",
                                description: "Encoder-decoder attention model",
                                parameters: "65M",
                                action: vec![(String::from("backtest"), ModelAction::MomBacktest(bt_mom())),(String::from("train"), ModelAction::MomTrain(train_mom()))],
                                on_result_bt: move |body: String| result_body_bt.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                                on_result_trn: move |body: String| result_body_trn.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                            }
                            ModelBlock { 
                                name: "DNN-8-8-8",
                                description: "Fully connected deep network",
                                parameters: "1.2M",
                                action: vec![(String::from("backtest"), ModelAction::MomBacktest(bt_mom())),(String::from("train"), ModelAction::MomTrain(train_mom()))],
                                on_result_bt: move |body: String| result_body_bt.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                                on_result_trn: move |body: String| result_body_trn.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                            }
                            ModelBlock { 
                                name: "CNN-16-32",
                                description: "Convolutional neural net",
                                parameters: "4.5M",
                                action: vec![(String::from("backtest"), ModelAction::MomBacktest(bt_mom())),(String::from("train"), ModelAction::MomTrain(train_mom()))],
                                on_result_bt: move |body: String| result_body_bt.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                                on_result_trn: move |body: String| result_body_trn.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                            }
                            ModelBlock { 
                                name: "LSTM-64",
                                description: "Long short-term memory net",
                                parameters: "850K",
                                action: vec![(String::from("backtest"), ModelAction::MomBacktest(bt_mom())),(String::from("train"), ModelAction::MomTrain(train_mom()))],
                                on_result_bt: move |body: String| result_body_bt.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                                on_result_trn: move |body: String| result_body_trn.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                            }
                            ModelBlock { 
                                name: "Transformer-Base",
                                description: "Encoder-decoder attention model",
                                parameters: "65M",
                                action: vec![(String::from("backtest"), ModelAction::MomBacktest(bt_mom())),(String::from("train"), ModelAction::MomTrain(train_mom()))],
                                on_result_bt: move |body: String| result_body_bt.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                                on_result_trn: move |body: String| result_body_trn.set(serde_json::from_str(body.as_str()).unwrap_or_default()),
                            }
                        }   
                    }
                }
            }
            section { class: "grid-wrapper",
                div { class: "grid-section-1x-center",
                    h3 {class: "qh3-term","Backtest"}
                    table { class: "kv-table",
                        tbody {
                            tr { th { "Sharpe" }              td { "{result_body_bt().sharpe}" } }
                            tr { th { "Sortino" }             td { "{result_body_bt().sortino}" } }
                            tr { th { "Max Drawdown" }        td { "{result_body_bt().mdd}" } }
                            tr { th { "T-Stat" }              td { "{result_body_bt().t_stat}" } }
                            tr { th { "Information Ratio" }   td { "{result_body_bt().information_ratio.unwrap_or(0.0)}" } }
                            tr { th { "Rows" }                td { "{result_body_bt().rows}" } }
                            tr { th { "Cols" }                td { "{result_body_bt().cols}" } }
                            tr { th { "Output Path" }         td { "{result_body_bt().path}" } }
                        }
                    }
                    table { class: "kv-table",
                        tbody {
                            tr { th{"μ (daily)"}     td{"{result_body_trn().test.mu_daily:.6e}"} }
                            tr { th{"σ (daily)"}     td{"{result_body_trn().test.sigma_daily:.6e}"} }
                            tr { th{"Sharpe (ann)"}  td{"{result_body_trn().test.sharpe_ann:.4}"} }
                            tr { th{"Avg position"}  td{"{result_body_trn().test.avg_position:.6}"} }
                            tr { th{"Avg turnover"}  td{"{result_body_trn().test.avg_turnover:.6e}"} }
                        }
                    }
                    div {
                        "Model Description with a citation if needed of the 
                        trading algorithm utilized in the model"
                    }
                }
                div { class: "grid-section-1x",
                    div { class: "grid-full-chart",
                        PlottersChart { 
                            matrix: data2.clone(),
                            y_axis : String::from("mean_price"), 
                        }
                    }
                }
            }
            section { class: "grid-wrapper",
                div { class: "grid-section-1x-center",
                    TradeDisplay { data: data.data.clone(), descrips: descrips }
                }
            }
            section {class: "grid-wrapper",
            div {
                "hello"
            }  }
        }
    }
}





#[component]
pub fn UiProvider(children: Element) -> Element {
    // Hooks at top level of the component are OK
    let start_date     = use_signal(|| None::<String>);
    let end_date       = use_signal(|| None::<String>);
    let instrument_ids = use_signal(|| vec![8147, 11667]);
    let bin_size       = use_signal(|| "5m".to_string());
    let url            = use_signal(|| "http://127.0.0.1:8080".to_string());
    let user           = use_signal(|| "root".to_string());
    let pass           = use_signal(|| "root".to_string());
    let ns             = use_signal(|| "equities".to_string());
    let db             = use_signal(|| "historical".to_string());
    let feats          = use_signal(|| Some("mom".to_string()));
    let model_path     = use_signal(|| Some("../ml-project/models/saved/test/final_model.onnx".to_string()));
    let out_csv        = use_signal(|| Some("../tmp_data/my_bt.csv".to_string()));
    let run_name       = use_signal(|| Some("test".to_string()));

    // The provider just captures already-created signals (no hooks inside)
    use_context_provider(|| UiCtx {
        start_date, end_date, instrument_ids, bin_size, url, user, pass, ns, db,
        feats, model_path, out_csv, run_name,
    });

    children
}



#[component]
fn ParamsForm() -> Element {
    let mut ctx = use_context::<UiCtx>();
    // local adapters for text <-> Vec<i64>
    let mut inst_text = use_signal({
        let ids = (ctx.instrument_ids)();
        move || ids.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
    });

    rsx! {
        div { class: "card",
        div {
            label { "Surreal URL" }
            input {
                value: "{(ctx.url)()}",
                oninput: move |ev| ctx.url.set(ev.value()),
            }
        }
        div{
            label { "User" }
            input {
                value: "{(ctx.user)()}",
                oninput: move |ev| ctx.user.set(ev.value()),
            }
        }
        div{
            label { "Pass" }
            input {
                r#type: "password",
                value: "{(ctx.pass)()}",
                oninput: move |ev| ctx.pass.set(ev.value()),
            }
        }
        div{
            label { "Namespace" }
            input {
                value: "{(ctx.ns)()}",
                oninput: move |ev| ctx.ns.set(ev.value()),
            }
        }
        div{
            label { "Database" }
            input {
                value: "{(ctx.db)()}",
                oninput: move |ev| ctx.db.set(ev.value()),
            }
        }
        div{
            label { "Bin size" }
            input {
                value: "{(ctx.bin_size)()}",
                oninput: move |ev| ctx.bin_size.set(ev.value()),
            }
        }
        div{
            label { "Instrument IDs (comma-sep.)" }
            input {
                value: "{inst_text()}",
                oninput: move |ev| {
                    let s = ev.value();
                    inst_text.set(s.clone());
                    // parse "8147,11667"
                    let parsed: Vec<i64> = s
                        .split(',')
                        .filter_map(|t| t.trim().parse::<i64>().ok())
                        .collect();
                    ctx.instrument_ids.set(parsed);
                }
            }
        }
        div{
            label { "Start date (ISO8601, optional)" }
            input {
                value: "{(ctx.start_date)().unwrap_or_default()}",
                oninput: move |ev| {
                    let v = ev.value();
                    ctx.start_date.set(if v.is_empty() { None } else { Some(v) });
                }
            }
        }
        div{
            label { "End date (ISO8601, optional)" }
            input {
                value: "{(ctx.end_date)().unwrap_or_default()}",
                oninput: move |ev| {
                    let v = ev.value();
                    ctx.end_date.set(if v.is_empty() { None } else { Some(v) });
                }
            }
        }
        div{
            label { "Run name (optional)" }
            input {
                value: "{(ctx.run_name)().unwrap_or_default()}",
                oninput: move |ev| {
                    let v = ev.value();
                    ctx.run_name.set(if v.is_empty() { None } else { Some(v) });
                }
            }
        }
        }
    }
}


#[derive(Props, PartialEq, Clone)]
pub struct ButtonDet {
    pub name: String,
    pub description: String,
    pub parameters: String,
    pub action: Vec<(String, ModelAction)>,
    #[props(default = EventHandler::new(|_| {}))]
    on_result_bt: EventHandler<String>,
    #[props(default = EventHandler::new(|_| {}))]
    on_result_trn: EventHandler<String>,
}

#[component]
pub fn ModelBlock(props: ButtonDet) -> Element {
    let ctx = use_context::<UiCtx>();
    let status = use_signal(|| RunStatus::Idle);
    let mut action = use_signal(|| true);
    let current = if action() { "train" } else { "backtest" }.to_string();
    rsx! {
        style { {include_str!("../css_files/model_button.css")}}
        button {
            class: "model-tile-btn",
            disabled: matches!(status(), RunStatus::Running),
            onclick: move |_| {
                let ctx = ctx.clone();
                let is_train = action(); 
                let mut status = status.to_owned();

                // clone actions you want to choose between (avoid borrowing props across await)
                let train_action = props.action[1].1.clone();
                let bt_action    = props.action[0].1.clone();

                // (optional) handlers if you need them
                let on_bt  = props.on_result_bt;
                let on_trn = props.on_result_trn;

                spawn(async move {
                    status.set(RunStatus::Running);
                    // Snapshot once per click (stable payload)
                    let ui = ctx.snapshot();
                    // Build generic request from action
                   // build exactly one spec based on the toggle (match version)
                    let spec = match is_train {
                        true  => ui.build_request(&train_action),
                        false => ui.build_request(&bt_action),
                    };
                    tracing::info!("{:?}",spec);
                    match execute_request(spec).await {
                        Ok(body) => {
                            if is_train {
                                tracing::info!("{:?}",body.clone());
                                on_trn.call(body.clone());
                            } else {
                                on_bt.call(body.clone());
                            }
                            status.set(RunStatus::Success(first_line(&body)));
                        }
                        Err(err) => {
                            let msg = err.to_string();
                            on_trn.call(msg.clone());
                            on_bt.call(msg.clone());
                            tracing::info!("{:?}",msg);
                            status.set(RunStatus::Error(msg));
                        }
                    }
                });
            },
            div { class: "model-tile-name", "{props.name}" }
            label {"Train"}
            select {
                value: "{current}",
                oninput: move |evt| {
                    let v = evt.value();
                    // map selected string to bool
                    println!("{}",v);
                    action.set(v == "train");
                },
                option { value: "train",    "Train" }
                option { value: "backtest", "Backtest" }
            }
            div { class: "model-tile-desc", "{props.description}" }
            div { class: "model-tile-stats", "Parameters: {props.parameters}" }
            match status() {
                RunStatus::Idle => rsx!( div { class: "model-tile-status", "Ready" } ),
                RunStatus::Running => rsx!( div { class: "model-tile-status running", "Running…" } ),
                RunStatus::Success(_msg) => rsx!( div { class: "model-tile-status ok", "✓ done" } ),
                RunStatus::Error(_err) => rsx!( div { class: "model-tile-status err", "✗ error" } ),
            }
        }
    }
}




#[derive(Props, PartialEq, Clone)]
pub struct SearchDropdownProps{
    /// Values to search/filter
    values: Vec<String>,
    /// Placeholder text
    #[props(default = "Search…".to_string())]
    placeholder: String,
    /// Callback fired when user selects a value
    onselect: EventHandler<String>,
}
