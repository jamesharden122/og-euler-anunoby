use serde::{Deserialize, Serialize};

struct SecurityData {
    pub instrument_id: u32,
    pub ts_in_delta: i32,
    pub size: i64,
    pub ts_recv: i64,
    pub price: i64,
}

impl Default for SecurityData {
    fn default() -> Self {
        SecurityData {
            instrument_id: 0,
            ts_in_delta: 0,
            size: 0,
            ts_recv: 0,
            price: 0,
        }
    }
}

impl SecurityData {
    // Function to set all security information
    pub fn set_security_info(
        &mut self,
        instrument_id: u32,
        ts_in_delta: i32,
        ts_recv: i64,
        size: i64,
        price: i64,
    ) {
        self.instrument_id = instrument_id;
        // Default values for sharpe, alpha, beta, and security score
        self.ts_in_delta = ts_in_delta;
        self.size = size;
        self.ts_recv = ts_recv;
        self.price = price;
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Hd {
    instrument_id: u32,
    length: u8,
    publisher_id: u32,
    rtype: u32,
    ts_event: u64,
}
