mod aggregator;
mod sensor_node;

use crate::aggregator::Aggregator;
use crate::sensor_node::SensorNode;
use tokio::task;

#[tokio::main]
async fn main() {
    println!("Starting Virtual IoT Network Simulation...");

    // 1. Start Aggregator on port 3030
    let aggregator = Aggregator::new(3030);
    let agg_handle = task::spawn(async move {
        aggregator.run().await;
    });

    // 2. Start Sensor Nodes
    let mut node_handles = vec![];
    for i in 1..=5 {
        let node_id = format!("sensor_{:03}", i);
        let node = SensorNode::new(node_id, "http://127.0.0.1:3030/telemetry".to_string(), 1);

        let handle = task::spawn(async move {
            // Give aggregator time to start
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            node.run().await;
        });
        node_handles.push(handle);
    }

    // Keep running
    let _ = tokio::join!(agg_handle);
}
