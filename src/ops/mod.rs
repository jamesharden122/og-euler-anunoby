pub mod datetimeops;
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use nalgebra::DMatrix;
#[cfg(feature = "server")]
use polars::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Props, PartialEq, Clone, Serialize, Deserialize)]
pub struct MyMatrix {
    #[serde(with = "matrix_as_vecvec")]
    #[props(default = DMatrix::zeros(10, 10))]
    pub data: DMatrix<f64>,
    pub colnames_enum: Option<Vec<(usize, String)>>,
    #[props(default = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0))]
    pub descrips: (f64, f64, f64, f64, f64, f64),
}

impl MyMatrix {
    // Constructor to create a new empty matrix
    pub fn new(rows: usize, cols: usize) -> Self {
        MyMatrix {
            data: DMatrix::zeros(rows, cols),
            descrips: (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            colnames_enum: None,
        }
    }

    pub fn new10x() -> Self {
        MyMatrix {
            data: DMatrix::zeros(10, 10),
            descrips: (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            colnames_enum: None,
        }
    }
    pub fn from(matrix: DMatrix<f64>) -> Self {
        MyMatrix {
            data: matrix,
            descrips: (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            colnames_enum: None,
        }
    }
    // Example method to add a scalar value to all elements of the matrix
    pub fn add_scalar(&mut self, value: f64) {
        self.data.iter_mut().for_each(|x| *x += value);
    }

    pub fn find_index(&self, target: &str) -> Option<usize> {
        self.colnames_enum
            .as_ref() // get Option<&Vec<_>>
            .and_then(|vec| vec.iter().find(|(_, name)| name == target).map(|(i, _)| *i))
    }
    #[cfg(feature = "server")]
    pub fn from_polars_dataframe(df: &DataFrame) -> PolarsResult<Self> {
        // Convert to 2D ndarray with row-major order
        let colnames_enum: Vec<(usize, String)> = df
            .get_column_names()
            .iter()
            .enumerate()
            .map(|(i, name)| (i, name.to_string()))
            .collect();
        println!("{:?}", colnames_enum);
        let ndarray = df.to_ndarray::<Float64Type>(IndexOrder::C)?;

        // Build DMatrix from the ndarray's raw slice
        let shape = ndarray.raw_dim(); // [nrows, ncols]
        let nrows = shape[0];
        let ncols = shape[1];

        let data = DMatrix::from_row_slice(nrows, ncols, ndarray.as_slice().unwrap());

        Ok(MyMatrix {
            data: data,
            descrips: (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            colnames_enum: Some(colnames_enum),
        })
    }
    // Method to print the dimensions of the matrix
    pub fn dimmensions(&self) {
        println!(
            "Matrix has {} rows and {} columns",
            self.data.nrows(),
            self.data.ncols()
        );
    }

    // Method to get a reference to the internal DMatrix
    pub fn inner_ref(&self) -> &DMatrix<f64> {
        &self.data
    }

    pub fn head(&self, shape: (usize, usize)) {
        let (num_rows, num_cols) = shape;

        // Get a view on the matrix
        let view = self.data.view((0, 0), (num_rows, num_cols));

        // Print the submatrix
        println!("Matrix View:\n{}", view);
    }

    pub fn scale_column(mut self, constant: f64, col: usize) -> Option<Self> {
        //let mut data = self.data.clone();
        self.data
            .column_mut(col)
            .iter_mut()
            .for_each(|x| *x *= constant);
        //let dt = task::spawn_blocking(move || { data // Return the modified data  }).await;
        //self.data = dt.unwrap();
        Some(self)
    }

    pub fn snapshot(&self, ind: usize) -> Option<(f64, f64, f64, f64, f64, f64)> {
        let col_means = self.data.row_mean();
        let standard_deviations = self.data.row_variance();
        Some((standard_deviations[ind], col_means[ind], 0.0, 0.0, 0.0, 0.0))
    }

    pub fn estimate_retuns(&self) -> Option<DMatrix<f64>> {
        // Create a new column for storing the returns with an initial NaN value for the first row
        let matrix = &self.data.clone();
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
