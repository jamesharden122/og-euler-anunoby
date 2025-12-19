#![cfg(feature = "server")] // Arc<Mutex<duckdb::Connection>>
use std::sync::{Arc, Mutex};
use tokio::sync::{OnceCell, RwLock};

pub type SharedDuck = Arc<Mutex<duckdb::Connection>>;

static DUCKSTORE: OnceCell<RwLock<Option<SharedDuck>>> = OnceCell::const_new();
pub mod duckstore {
    use super::*;
    async fn handle() -> &'static RwLock<Option<SharedDuck>> {
        DUCKSTORE.get_or_init(|| async { RwLock::new(None) }).await
    }
    pub async fn set(conn: SharedDuck) {
        *handle().await.write().await = Some(conn);
    }
    pub async fn get() -> Option<SharedDuck> {
        handle().await.read().await.clone()
    }
    pub async fn clear() {
        *handle().await.write().await = None;
    }
}
