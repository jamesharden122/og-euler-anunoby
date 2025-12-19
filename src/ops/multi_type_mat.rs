use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use nalgebra::DMatrix;
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

    pub fn find_index(&self, target: &str) -> Option<usize> {
        self.colnames_enum_f64
            .as_ref()
            .and_then(|vec| vec.iter().find(|(_, name)| name == target).map(|(i, _)| *i))
            .or_else(|| {
                self.colnames_enum_str
                    .as_ref()
                    .and_then(|vec| vec.iter().find(|(_, name)| name == target).map(|(i, _)| *i))
            })
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
