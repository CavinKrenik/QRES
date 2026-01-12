use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use warp::Filter;

/// Data payload received from sensors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorData {
    pub id: String,
    pub value: f64,
    pub timestamp: u64,
}

/// In-memory store for the latest sensor readings.
type Store = Arc<Mutex<HashMap<String, SensorData>>>;

/// The central aggregation server.
pub struct Aggregator {
    port: u16,
    store: Store,
}

impl Aggregator {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run(&self) {
        let store = self.store.clone();
        let store_filter = warp::any().map(move || store.clone());

        // POST /telemetry: Receive data from sensors
        let telemetry = warp::post()
            .and(warp::path("telemetry"))
            .and(warp::body::json())
            .and(store_filter.clone())
            .map(|data: SensorData, store: Store| {
                let mut map = store.lock().unwrap();
                map.insert(data.id.clone(), data);
                warp::reply::json(&"ok")
            });

        // GET /metrics: Return current state of all sensors
        let metrics = warp::get()
            .and(warp::path("metrics"))
            .and(store_filter.clone())
            .map(|store: Store| {
                let map = store.lock().unwrap();
                warp::reply::json(&*map)
            });

        // Determine static path based on CWD
        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let static_dir = if current_dir.join("static").exists() {
            "static".to_string()
        } else {
            "examples/virtual_iot_network/static".to_string()
        };
        let index_path = format!("{}/index.html", static_dir);

        println!(">> Serving static files from: {}", static_dir);

        // GET / -> Serve index.html explicitly
        let root = warp::get()
            .and(warp::path::end())
            .and(warp::fs::file(index_path));

        // GET /... -> Serve other static files
        let static_files = warp::fs::dir(static_dir);

        // Order matters: specific paths first, then root, then generic static fallthrough
        let routes = telemetry.or(metrics).or(root).or(static_files);

        println!(
            ">> Hive Aggregator online at http://127.0.0.1:{}",
            self.port
        );
        warp::serve(routes).run(([127, 0, 0, 1], self.port)).await;
    }
}
