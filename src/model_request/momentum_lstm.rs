// src/model_block.rs
#![allow(non_snake_case)]
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::debug;
use urlencoding::encode;

// ==========================
// 0) HTTP (web-only helpers)
// ==========================

// ==========================================
// 2) Action types you can extend per model
// ==========================================
#[derive(Clone, Debug, PartialEq)]
pub struct TrainSpec {
    /// Name/path of your trainer class and scripts; you can generalize later
    pub trainer_path: String, // e.g. "../ml-project/models/mls_lstm_trainer.py"
    pub trainer_class: String, // e.g. "MLSLSTMTrainer"
    pub time_steps: usize,
    pub input_dim: usize,
    pub val_split: f32,
    pub test_split: f32,
    pub epochs: usize,
    pub batch_size: Option<usize>,
    pub verbose: u8,
    pub shuffle_before_split: bool,
    pub seed: u64,
    pub save_every_epoch: bool,
    pub save_weights_only: bool,
    pub monitor: String, // e.g. "val_loss"
    pub save_best_only: bool,
    /// Write/Load knobs that often vary between models
    pub feature_names: serde_json::Value, // e.g. [{"MomFactor": null}]
    pub feature_spec: serde_json::Value, // e.g. {"mean_price":"float32",...}
    pub tfrecord_out: String,            // e.g. "../tmp_data/mom_data.tfrecord"
    pub writer_path: String,             // e.g. "../ml-project/py/pl2tfrecord_writer.py"
    pub reader_path: String,             // e.g. "../ml-project/py/pl2tfrecord_reader.py"
    pub sequence_length: usize,
    pub horizon: usize,
    pub stride: usize,
    pub group_col: String,  // "instrument_id"
    pub return_col: String, // "ret"
    pub sigma_col: String,  // "sigma"
    pub cost_col: String,   // "cost"
    pub include_cost: bool,
    pub gzip: bool,
    pub shuffle: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BacktestSpec {
    pub feats: String,      // e.g. "mom"
    pub model_path: String, // path to .onnx
    pub out_csv: String,    // where to write
}

impl super::BuildRequest for super::UiParams {
    fn build_request(&self, action: &super::ModelAction) -> super::RequestSpec {
        match action {
            super::ModelAction::MomTrain(spec) => {
                // Build inner req_json (your backend expects "req_json": {...})
                let write = serde_json::json!({
                    "column_set": ["instrument_id","bin","t0","t1","mean_price","ret"],
                    "srt": ["instrument_id","bin"],
                    "exclude_cols": ["instrument_id","bin","t0","t1","ret","sigma","cost"],
                    "query_params": [self.bin_size, self.instrument_ids],
                    "out_path": spec.tfrecord_out,
                    "attr": "write_timeseries_tfrecord_for_sharpe_from_polars",
                    "writer_path": spec.writer_path,
                    "feature_names": spec.feature_names,   // e.g. [{"MomFactor": null}]
                    "target_col": serde_json::Value::Null,
                    "time_col": serde_json::Value::Null,
                    "sequence_length": spec.sequence_length,
                    "horizon": spec.horizon,
                    "stride": spec.stride,
                    "group_col": spec.group_col,
                    "compress": true,
                    "return_col": spec.return_col,
                    "sigma_col": spec.sigma_col,
                    "cost_col": spec.cost_col
                });

                let load = serde_json::json!({
                    "reader_py_path": spec.reader_path,
                    "tfrecord_paths": [spec.tfrecord_out],
                    "attr": "load_sharpe_timeseries_dataset",
                    "feature_spec": spec.feature_spec,
                    "label": serde_json::Value::Null,
                    "label_dtype": serde_json::Value::Null,
                    "batch_size": spec.batch_size.unwrap_or(5),
                    "shuffle": spec.shuffle,
                    "gzip": spec.gzip,
                    "include_cost": spec.include_cost
                });

                let train = serde_json::json!({
                    "trainer_path": spec.trainer_path,
                    "class": spec.trainer_class,
                    "attr": "train",
                    "time_steps": spec.time_steps,
                    "input_dim": spec.input_dim,
                    "val_split": spec.val_split,
                    "test_split": spec.test_split,
                    "epochs": spec.epochs,
                    "batch_size": spec.batch_size, // can be null
                    "verbose": spec.verbose,
                    "shuffle_before_split": spec.shuffle_before_split,
                    "seed": spec.seed,
                    "save_every_epoch": spec.save_every_epoch,
                    "save_weights_only": spec.save_weights_only,
                    "monitor": spec.monitor,
                    "save_best_only": spec.save_best_only,
                    "run_name": self.run_name.clone().unwrap_or_else(|| "run".into()),
                });

                let req_json = serde_json::json!({
                    "db": { "url": self.db.url, "user": self.db.user, "pass": self.db.pass, "ns": self.db.ns, "dbname": self.db.db },
                    "write": write,
                    "load": load,
                    "train": train
                });
                let base = self.db.url.trim_end_matches('/');
                let url = format!("{}/tsmomnn/train?req_json={}", base, &req_json.to_string());
                super::RequestSpec {
                    method: super::HttpMethod::Get,
                    url,
                    body_json: None,
                    headers: vec![
                        ("Accept".to_string(), "application/json".to_string()),
                        ("Content-Type".to_string(), "application/json".to_string()),
                    ], // default JSON header auto-added
                }
            }

            super::ModelAction::MomBacktest(spec) => {
                let insts = self
                    .instrument_ids
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                let url = format!(
                    "{}/tsmomnn/backtest?inst_ids={}&bin_size={}&model_path={}&feats={}&out_csv={}",
                    self.db.url.trim_end_matches('/'),
                    urlencoding::encode(&insts),
                    urlencoding::encode(&self.bin_size),
                    urlencoding::encode(&spec.model_path),
                    urlencoding::encode(&spec.feats),
                    urlencoding::encode(&spec.out_csv),
                );
                debug!("{:?}", url);
                super::RequestSpec {
                    method: super::HttpMethod::Get,
                    url,
                    body_json: None,
                    headers: vec![("Accept".to_string(), "application/json".to_string())],
                }
            }
        }
    }
}
