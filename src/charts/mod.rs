pub mod candle_stick;
pub mod clustering;
pub mod single_asset_lc;

pub enum ChartType {
    Line,
    Candle,
}

impl ChartType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChartType::Line => "line",
            ChartType::Candle => "candle",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "line" => ChartType::Line,
            "candle" => ChartType::Candle,
            _ => ChartType::Line,
        }
    }
    pub fn from_usize_as_str(s: usize) -> &'static str {
        match s {
            0 => "line",
            1 => "candle",
            _ => "None",
        }
    }
}
