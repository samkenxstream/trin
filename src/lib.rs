use discovery::{Config as DiscoveryConfig, Discovery, Enr};
use discv5::Discv5ConfigBuilder;
use log::{error, info};
use parking_lot::RwLock;
use std::{collections::HashMap, time::Duration};
use tokio::time::sleep;
mod alexandria;
pub mod discovery;

pub const DISCOVERY_INTERVAL: u64 = 30;
pub const QUERY_INTERVAL: u64 = 30;

fn talk_resp(_protocol: &[u8], req: &[u8]) -> Vec<u8> {
    return req.to_vec();
}

pub struct Service {
    discovery: Discovery,
    db: RwLock<HashMap<String, Vec<u8>>>,
}

impl Service {
    pub async fn new(port: u16, boot_nodes: Vec<Enr>) -> Self {
        let disv5_config = Discv5ConfigBuilder::default()
            .talkreq_callback(talk_resp)
            .build();

        let mut config = DiscoveryConfig::default();
        config.discv5_config = disv5_config;
        config.listen_port = port;
        config.bootnode_enrs = boot_nodes;

        let discovery = Discovery::new(config).await.unwrap();
        Self {
            discovery,
            db: RwLock::new(HashMap::new()),
        }
    }

    /// Make a FIND_NODE query every `DISCOVERY_INTERVAL` and add the discovered nodes
    /// into the routing table
    pub async fn find_peers_loop(&mut self) {
        loop {
            info!(
                "Number of connected peers: {}",
                self.discovery.connected_peers()
            );
            match self.discovery.discover_nodes().await {
                Ok(_) => info!("Completed find_node query"),
                Err(e) => {
                    error!("Error finding peers: {}", e);
                    break;
                }
            }
            sleep(Duration::from_secs(DISCOVERY_INTERVAL)).await;
        }
    }

    /// Make requests for random keys every `QUERY_INTERVAL` seconds
    pub async fn get_value(&mut self) {
        loop {}
    }
}
