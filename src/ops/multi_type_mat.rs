use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use linfa::dataset::DatasetBase;
use linfa::traits::{Fit, Predict};
use linfa_clustering::{KMeans, KMeansInit};
use linfa_reduction::Pca;
use nalgebra::DMatrix;
use ndarray::{Array2, ArrayBase, Data, Ix2};
#[cfg(feature = "server")]
use polars::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Props, PartialEq, Clone, Serialize, Deserialize)]
pub struct MyMmMatrix {
    // Numeric matrix and column names
    #[serde(with = "matrix_as_vecvec")]
    #[props(default = DMatrix::zeros(10, 10))]
    pub data_f64: DMatrix<f64>,
    pub colnames_enum_f64: Option<Vec<(usize, String)>>,

    // String matrix and column names (explicit request: use DMatrix<String>)
    // Skipping serde by default to avoid cross-target friction
    #[serde(with = "matrix_string_as_vecvec")]
    #[props(default = DMatrix::from_element(0, 0, String::new()))]
    pub data_str: DMatrix<String>,
    pub colnames_enum_str: Option<Vec<(usize, String)>>,

    // Descriptive statistics for numeric data only
    #[props(default = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0))]
    pub descrips: (f64, f64, f64, f64, f64, f64),
}

impl MyMmMatrix {
    // Constructor to create a new empty matrix
    pub fn new(rows: usize, cols: usize) -> Self {
        MyMmMatrix {
            data_f64: DMatrix::zeros(rows, cols),
            descrips: (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            colnames_enum_f64: None,
            data_str: DMatrix::from_element(0, 0, String::new()),
            colnames_enum_str: None,
        }
    }

    pub fn new10x() -> Self {
        MyMmMatrix {
            data_f64: DMatrix::zeros(10, 10),
            descrips: (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            colnames_enum_f64: None,
            data_str: DMatrix::from_element(0, 0, String::new()),
            colnames_enum_str: None,
        }
    }
    pub fn from(matrix: DMatrix<f64>) -> Self {
        MyMmMatrix {
            data_f64: matrix,
            descrips: (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            colnames_enum_f64: None,
            data_str: DMatrix::from_element(0, 0, String::new()),
            colnames_enum_str: None,
        }
    }
    // Example method to add a scalar value to all elements of the matrix
    pub fn add_scalar(&mut self, value: f64) {
        self.data_f64.iter_mut().for_each(|x| *x += value);
    }

    pub fn find_index_f64(&self, target: &str) -> Option<usize> {
        self.colnames_enum_f64
            .as_ref()
            .and_then(|vec| vec.iter().find(|(_, name)| name == target).map(|(i, _)| *i))
    }

    pub fn find_index_str(&self, target: &str) -> Option<usize> {
        self.colnames_enum_str
            .as_ref()
            .and_then(|vec| vec.iter().find(|(_, name)| name == target).map(|(i, _)| *i))
    }

    pub fn find_col_index_str(&self, col_ind: &str, target: &str) -> Option<usize> {
        todo!()
    }

    pub fn find_col_index_f64(&self, col_ind: &str, target: &str) -> Option<usize> {
        todo!()
    }

    #[cfg(feature = "server")]
    pub fn from_polars_dataframe(df: &mut DataFrame) -> PolarsResult<Self> {
        //split up dataframe colnames by type
        let mut str_nm = Vec::new();
        let mut float_nm = Vec::new();
        let schema_clone = df.schema().clone();
        for val in schema_clone.iter() {
            match val.1 {
                DataType::String => str_nm.push(val.0.as_str()),
                DataType::Date => {
                    str_nm.push(val.0.as_str());
                    df.try_apply("date", |s| s.cast(&DataType::String))?;
                }
                _ => float_nm.push(val.0.as_str()),
            }
        }
        let df_str = &df.select(str_nm.clone())?;
        let df_float = &df.select(float_nm.clone())?;
        println!("{:?}", df_str.shape());
        println!("{:?}", df_float.shape());

        let tup_str_nm: Vec<(usize, String)> = str_nm
            .iter_mut()
            .map(|nm| (df_str.get_column_index(nm).unwrap(), nm.to_string()))
            .collect();
        let tup_float_nm: Vec<(usize, String)> = float_nm
            .iter_mut()
            .map(|nm| (df_float.get_column_index(nm).unwrap(), nm.to_string()))
            .collect();
        println!(
            "String DataFrame tuple(index, column_name):{:?}",
            tup_str_nm.clone()
        );
        println!(
            "Float DataFrame tuple(index, column_name):{:?}",
            tup_float_nm.clone()
        );

        let ndarray_float = df_float.to_ndarray::<datatypes::Float64Type>(IndexOrder::C)?;
        let nrows_str = df_str.height();
        let ncols_str = df_str.width();

        let mut array_str: Vec<String> = Vec::with_capacity(nrows_str * ncols_str);
        for s in df_str.iter() {
            let ca = s.str()?; // StringChunked
            if ca.null_count() == 0 {
                // Fast path: Iterator<Item = &str> :contentReference[oaicite:2]{index=2}
                array_str.extend(ca.into_no_null_iter().map(str::to_owned));
            } else {
                // 2) fill nulls with empty string (or something else)
                array_str.extend(ca.into_iter().map(|opt| opt.unwrap_or("").to_owned()));
            }
        }
        // Build DMatrix from the ndarray's raw slice
        let shape_float = ndarray_float.raw_dim(); // [nrows, ncols]
        let nrows_float = shape_float[0];
        let ncols_float = shape_float[1];

        let mut data_f64 =
            DMatrix::from_row_slice(nrows_float, ncols_float, ndarray_float.as_slice().unwrap());
        // Sanitize any non-finite floats (NaN/Inf) to ensure JSON/wasm transport compatibility
        for v in data_f64.iter_mut() {
            if !v.is_finite() {
                *v = 0.0;
            }
        }
        let data_str = DMatrix::from_vec(nrows_str, ncols_str, array_str);

        Ok(MyMmMatrix {
            data_f64,
            descrips: (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            colnames_enum_f64: Some(tup_float_nm),
            data_str,
            colnames_enum_str: Some(tup_str_nm),
        })
    }
    // Method to print the dimensions of the matrix
    pub fn dimmensions(&self) {
        println!(
            "Matrix has {} rows and {} columns",
            self.data_f64.nrows(),
            self.data_f64.ncols()
        );
    }

    // Method to get a reference to the internal DMatrix
    pub fn inner_ref(&self) -> &DMatrix<f64> {
        &self.data_f64
    }

    pub fn head(&self, shape: (usize, usize)) {
        let (num_rows, num_cols) = shape;

        // Get a view on the matrix
        let view = self.data_f64.view((0, 0), (num_rows, num_cols));

        // Print the submatrix
        println!("Matrix View:\n{}", view);
    }

    pub fn scale_column(mut self, constant: f64, col: usize) -> Option<Self> {
        //let mut data = self.data.clone();
        self.data_f64
            .column_mut(col)
            .iter_mut()
            .for_each(|x| *x *= constant);
        //let dt = task::spawn_blocking(move || { data // Return the modified data  }).await;
        //self.data = dt.unwrap();
        Some(self)
    }

    pub fn snapshot(&self, ind: usize) -> Option<(f64, f64, f64, f64, f64, f64)> {
        let col_means = self.data_f64.row_mean();
        let standard_deviations = self.data_f64.row_variance();
        Some((standard_deviations[ind], col_means[ind], 0.0, 0.0, 0.0, 0.0))
    }

    pub fn estimate_retuns(&self) -> Option<DMatrix<f64>> {
        // Create a new column for storing the returns with an initial NaN value for the first row
        let matrix = &self.data_f64.clone();
        let rows = matrix.shape().0;
        let cols = matrix.shape().1;
        let mut returns_column = vec![0.0; rows];

        // Calculate the first difference and returns for the first column
        for i in 1..rows {
            let previous_value = matrix[(i - 1, 0)];
            let current_value = matrix[(i, 0)];
            let first_diff = current_value - previous_value;

            if previous_value != 0.0 {
                returns_column[i] = first_diff / previous_value;
            }
        }

        // Create a new matrix with 5 columns to accommodate the original data plus the returns column
        let mut extended_matrix = DMatrix::from_element(rows, cols + 1, 0.0);

        // Copy the original matrix data into the new extended matrix
        for r in 0..rows {
            for c in 0..cols {
                extended_matrix[(r, c)] = matrix[(r, c)];
            }
        }

        // Add the returns column as the fifth column in the new matrix
        for r in 0..rows {
            extended_matrix[(r, cols)] = returns_column[r];
        }
        Some(extended_matrix)
    }
    pub fn convert_nano_to_datetime(ts_recv: f64) -> Option<DateTime<Utc>> {
        let seconds = (ts_recv / 1_000_000_000.0) as i64;
        let nanoseconds = (ts_recv % 1_000_000_000.0) as u32;

        // Create a NaiveDateTime from ts_recv
        let datetime_recv = DateTime::<Utc>::from_timestamp(seconds, nanoseconds).unwrap();
        println!("Received DateTime: {}", datetime_recv);
        Some(datetime_recv)
    }

    /// Convert nalgebra DMatrix (column-major) -> ndarray Array2 (row-major) safely.
    fn dmatrix_to_array2(x: &DMatrix<f64>) -> Array2<f64> {
        let (n, d) = (x.nrows(), x.ncols());
        Array2::from_shape_fn((n, d), |(i, j)| x[(i, j)])
    }

    /// Convert ndarray Array2 -> nalgebra DMatrix
    fn array2_to_dmatrix(a: &Array2<f64>) -> DMatrix<f64> {
        let (n, d) = a.dim();
        DMatrix::from_fn(n, d, |i, j| a[(i, j)])
    }

    /// Fit PCA on a DMatrix where rows = samples, cols = features.
    /// Returns (fitted_model, projected_data_as_DMatrix).
    pub fn pca_fit_transform_dmatrix(
        mut x: DMatrix<f64>,
        k: usize,
        col_drop_idxs: Option<Vec<usize>>,
    ) -> Result<(Array2<f64>, Array2<f64>), String> {
        tracing::debug!("Removing Columns");
        let drop_idxs = col_drop_idxs.as_deref().unwrap_or(&[]);
        if !drop_idxs.is_empty() {
            x = x.remove_columns_at(drop_idxs);
        }

        let mut records = Self::dmatrix_to_array2(&x);
        tracing::debug!("Replacing NaNs with 0's");
        records.mapv_inplace(|v| if v.is_finite() { v } else { 0.0 });

        let n_samples = records.nrows();
        let n_features = records.ncols();
        if n_samples < 2 || n_features == 0 {
            return Ok((Array2::zeros((0, 0)), Array2::zeros((0, 0))));
        }

        // Drop near-constant columns to avoid zero-norm directions in the solver.
        let mut keep: Vec<usize> = Vec::with_capacity(n_features);
        for j in 0..n_features {
            let mean = records.column(j).sum() / (n_samples as f64);
            let mut ss = 0.0_f64;
            for i in 0..n_samples {
                let diff = records[(i, j)] - mean;
                ss += diff * diff;
            }
            let var = ss / (n_samples as f64);
            if var.is_finite() && var > 1e-12 {
                keep.push(j);
            }
        }

        if keep.is_empty() {
            return Ok((
                Array2::zeros((n_samples, 0)),
                Array2::zeros((0, n_features)),
            ));
        }

        let mut reduced = Array2::zeros((n_samples, keep.len()));
        for (new_j, &old_j) in keep.iter().enumerate() {
            for i in 0..n_samples {
                reduced[(i, new_j)] = records[(i, old_j)];
            }
        }

        // After mean-centering, the maximum rank is `min(n_features, n_samples - 1)`.
        let k_max = reduced.ncols().min(n_samples.saturating_sub(1));
        if k_max == 0 {
            return Ok((Array2::zeros((0, 0)), Array2::zeros((0, 0))));
        }
        let k = k.clamp(1, k_max);

        // On wasm32, `linfa_reduction::Pca` can panic inside `linfa-linalg` on some ill-conditioned
        // inputs. Use a small, deterministic PCA implementation (covariance + symmetric eigendecomp)
        // instead, to avoid bringing down the whole app.
        let (mut scores, components_reduced): (Array2<f64>, Array2<f64>) = {
            #[cfg(target_arch = "wasm32")]
            {
                let d_reduced = reduced.ncols();
                let mut centered = reduced;

                // Mean-center and standardize each column to improve conditioning.
                let mut means = vec![0.0_f64; d_reduced];
                for j in 0..d_reduced {
                    means[j] = centered.column(j).sum() / (n_samples as f64);
                }
                for i in 0..n_samples {
                    for j in 0..d_reduced {
                        centered[(i, j)] -= means[j];
                    }
                }

                let mut stds = vec![1.0_f64; d_reduced];
                for j in 0..d_reduced {
                    let mut ss = 0.0_f64;
                    for i in 0..n_samples {
                        let v = centered[(i, j)];
                        ss += v * v;
                    }
                    let var = ss / (n_samples as f64 - 1.0).max(1.0);
                    let std = var.sqrt();
                    stds[j] = if std.is_finite() && std > 1e-12 {
                        std
                    } else {
                        1.0
                    };
                }
                for i in 0..n_samples {
                    for j in 0..d_reduced {
                        centered[(i, j)] /= stds[j];
                    }
                }

                // Covariance matrix (d x d), symmetrized for numerical stability.
                let denom = (n_samples as f64 - 1.0).max(1.0);
                let mut cov = centered.t().dot(&centered);
                cov.mapv_inplace(|v| (v / denom).clamp(-f64::MAX, f64::MAX));
                cov.mapv_inplace(|v| if v.is_finite() { v } else { 0.0 });
                let mut cov = cov.as_standard_layout().to_owned();
                for i in 0..d_reduced {
                    for j in 0..i {
                        let v = 0.5 * (cov[(i, j)] + cov[(j, i)]);
                        cov[(i, j)] = v;
                        cov[(j, i)] = v;
                    }
                }

                let cov_slice = cov
                    .as_slice()
                    .ok_or_else(|| "Covariance matrix is not contiguous".to_string())?;
                let cov_dm = DMatrix::from_row_slice(d_reduced, d_reduced, cov_slice);
                let eig = nalgebra::linalg::SymmetricEigen::new(cov_dm);

                if eig.eigenvalues.iter().any(|v| !v.is_finite()) {
                    return Err("PCA failed: non-finite eigenvalues".to_string());
                }

                let mut idx: Vec<usize> = (0..d_reduced).collect();
                idx.sort_by(|&a, &b| {
                    let va = eig.eigenvalues[a];
                    let vb = eig.eigenvalues[b];
                    // Sort descending, and push any non-finite values to the end.
                    let va = if va.is_finite() {
                        va
                    } else {
                        f64::NEG_INFINITY
                    };
                    let vb = if vb.is_finite() {
                        vb
                    } else {
                        f64::NEG_INFINITY
                    };
                    vb.total_cmp(&va)
                });

                let mut components_reduced = Array2::zeros((k, d_reduced));
                for (comp_i, &eig_idx) in idx.iter().take(k).enumerate() {
                    for feat_j in 0..d_reduced {
                        components_reduced[(comp_i, feat_j)] = eig.eigenvectors[(feat_j, eig_idx)];
                    }
                }

                let scores = centered.dot(&components_reduced.t());
                (scores, components_reduced)
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                tracing::debug!("Creating Linfa Database");
                let ds = DatasetBase::new(reduced, ()); // no targets
                tracing::debug!("Fitting PCA");
                let model = Pca::params(k)
                    .fit(&ds)
                    .map_err(|e| format!("PCA fit failed: {e:?}"))?;
                tracing::debug!("Estimating Scores");
                let embedded = model.predict(ds);
                let scores: Array2<f64> = embedded.records().as_standard_layout().to_owned();
                let components_reduced: Array2<f64> =
                    model.components().as_standard_layout().to_owned();
                (scores, components_reduced)
            }
        };

        let mut components = Array2::zeros((k, n_features));
        for (new_j, &old_j) in keep.iter().enumerate() {
            for i in 0..k {
                components[(i, old_j)] = components_reduced[(i, new_j)];
            }
        }

        tracing::debug!("After PCA making sure the scores and components are finite.");
        scores.mapv_inplace(|v| if v.is_finite() { v } else { 0.0 });
        components.mapv_inplace(|v| if v.is_finite() { v } else { 0.0 });

        Ok((scores, components))
    }

    pub fn kmeans_clusters(x: DMatrix<f64>) -> Result<Array2<f64>, String> {
        let n_samples = x.nrows();
        let n_features = x.ncols();
        if n_samples == 0 {
            return Ok(Array2::zeros((0, 1)));
        }
        if n_features == 0 {
            return Ok(Array2::zeros((n_samples, 1)));
        }
        tracing::debug!("Replacing NaNs with 0's");
        let mut records = Self::dmatrix_to_array2(&x);
        records.mapv_inplace(|v| if v.is_finite() { v } else { 0.0 });
        // Standardize each feature so L2 distance doesn't get dominated by scale.
        let mut means = vec![0.0_f64; n_features];
        for j in 0..n_features {
            means[j] = records.column(j).sum() / (n_samples as f64);
        }

        let mut stds = vec![1.0_f64; n_features];
        for j in 0..n_features {
            let mut ss = 0.0_f64;
            for i in 0..n_samples {
                let diff = records[(i, j)] - means[j];
                ss += diff * diff;
            }
            let var = ss / (n_samples as f64);
            let std = var.sqrt();
            stds[j] = if std.is_finite() && std > 1e-12 {
                std
            } else {
                1.0
            };
        }
        tracing::debug!("Standardize Columns");
        for i in 0..n_samples {
            for j in 0..n_features {
                records[(i, j)] = (records[(i, j)] - means[j]) / stds[j];
            }
        }

        // Choose k heuristically since the signature doesn't accept it.
        let k = if n_samples <= 1 {
            1
        } else {
            let k_max = n_samples.min(8);
            ((n_samples as f64).sqrt().round() as usize).clamp(2, k_max)
        };
        tracing::debug!("Creating Linfa Database");
        let ds = DatasetBase::new(records, ());
        tracing::debug!("Instantiate Kmeans");
        // When targeting wasm (Dioxus web), `getrandom` is already wired up (via `getrandom/js`),
        // so we can use it to pick randomized initial centroids without pulling in `rand` here.
        let init = KMeansInit::KMeansPlusPlus;
        tracing::debug!("Estimate groups");
        let model = KMeans::params(k)
            .init_method(init)
            .tolerance(1e-2)
            .fit(&ds)
            .map_err(|e| format!("KMeans fit failed: {e:?}"))?;
        let clustered = model.predict(ds);
        let memberships = clustered.targets();
        tracing::debug!("Output Labels");
        Ok(Array2::from_shape_fn((n_samples, 1), |(i, _)| {
            memberships[i] as f64
        }))
    }
}

mod matrix_as_vecvec {
    use super::*;
    use serde::de::{SeqAccess, Visitor};
    use serde::ser::SerializeSeq;
    use serde::{Deserializer, Serializer};
    use std::fmt;

    pub fn serialize<S>(matrix: &DMatrix<f64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let rows: Vec<Vec<f64>> = (0..matrix.nrows())
            .map(|i| matrix.row(i).iter().copied().collect())
            .collect();
        rows.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DMatrix<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MatrixVisitor;

        impl<'de> Visitor<'de> for MatrixVisitor {
            type Value = DMatrix<f64>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a 2D array of floats")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<DMatrix<f64>, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut data = Vec::new();
                let mut row_len = None;

                while let Some(row) = seq.next_element::<Vec<f64>>()? {
                    if let Some(len) = row_len {
                        if row.len() != len {
                            return Err(serde::de::Error::custom("inconsistent row length"));
                        }
                    } else {
                        row_len = Some(row.len());
                    }
                    data.extend(row);
                }

                let cols = row_len.unwrap_or(0);
                let rows = data.len() / cols;
                Ok(DMatrix::from_row_slice(rows, cols, &data))
            }
        }

        deserializer.deserialize_seq(MatrixVisitor)
    }
}

// Optional serde helpers for a DMatrix<String>.
// If you later want to serialize/deserialize `data_str`,
// swap its `#[serde(skip)]` for `#[serde(with = "matrix_string_as_vecvec")]`.
#[allow(dead_code)]
mod matrix_string_as_vecvec {
    use super::*;
    use serde::de::{SeqAccess, Visitor};
    use serde::ser::SerializeSeq;
    use serde::{Deserializer, Serializer};
    use std::fmt;

    pub fn serialize<S>(matrix: &DMatrix<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let rows: Vec<Vec<String>> = (0..matrix.nrows())
            .map(|i| matrix.row(i).iter().cloned().collect())
            .collect();
        rows.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DMatrix<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MatrixVisitor;

        impl<'de> Visitor<'de> for MatrixVisitor {
            type Value = DMatrix<String>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a 2D array of strings")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<DMatrix<String>, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut data: Vec<String> = Vec::new();
                let mut row_len: Option<usize> = None;

                while let Some(row) = seq.next_element::<Vec<String>>()? {
                    if let Some(len) = row_len {
                        if row.len() != len {
                            return Err(serde::de::Error::custom("inconsistent row length"));
                        }
                    } else {
                        row_len = Some(row.len());
                    }
                    data.extend(row.into_iter());
                }

                let cols = row_len.unwrap_or(0);
                let rows = if cols == 0 { 0 } else { data.len() / cols };
                Ok(DMatrix::from_row_slice(rows, cols, &data))
            }
        }

        deserializer.deserialize_seq(MatrixVisitor)
    }
}
