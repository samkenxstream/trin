use super::{
    discovery::{Config as DiscoveryConfig, Discovery},
    types::{FindContent, FindNodes, FoundContent, Nodes, Ping, Pong, Request, Response},
    U256,
};
use super::{types::Message, Enr};
use discv5::{Discv5ConfigBuilder, TalkReqHandler};

pub const PROTOCOL: &str = "state-network";

pub struct AlexandriaProtocol {
    discovery: Discovery,
    data_radius: U256,
}

impl TalkReqHandler for AlexandriaProtocol {
    fn talkreq_response(&self, protocol: &[u8], req: &[u8]) -> Vec<u8> {
        if let Ok(protocol) = String::from_utf8(protocol.to_vec()) {
            if protocol != PROTOCOL {
                return "INVALID PROTOCOL".as_bytes().to_vec();
            }
            match Message::from_bytes(req) {
                Err(e) => format!("INVALID REQUEST: {}", e).as_bytes().to_vec(),
                Ok(msg) => {
                    if let Message::Request(req) = msg {
                        Message::Response(self.handle_request(req)).to_bytes()
                    } else {
                        "INVALID MESSAGE".as_bytes().to_vec()
                    }
                }
            }
        } else {
            "INVALID PROTOCOL".as_bytes().to_vec()
        }
    }
}

impl AlexandriaProtocol {
    pub async fn new(port: u16, boot_nodes: Vec<Enr>, data_radius: U256) -> Self {
        let disv5_config = Discv5ConfigBuilder::default().build();

        let mut config = DiscoveryConfig::default();
        config.discv5_config = disv5_config;
        config.listen_port = port;
        config.bootnode_enrs = boot_nodes;

        let discovery = Discovery::new(config, None).await.unwrap();
        Self {
            discovery,
            data_radius,
        }
    }

    pub async fn send_ping(
        &self,
        data_radius: U256,
        enr_seq: u32,
        enr: Enr,
    ) -> Result<Vec<u8>, String> {
        let msg = Ping {
            data_radius,
            enr_seq,
        };
        self.discovery
            .send_talkreq(enr, Message::Request(Request::Ping(msg)).to_bytes())
            .await
    }

    pub async fn send_find_nodes(&self, distances: Vec<u64>, enr: Enr) -> Result<Vec<u8>, String> {
        let msg = FindNodes { distances };
        self.discovery
            .send_talkreq(enr, Message::Request(Request::FindNodes(msg)).to_bytes())
            .await
    }

    pub async fn send_find_content(
        &self,
        content_key: Vec<u8>,
        enr: Enr,
    ) -> Result<Vec<u8>, String> {
        let msg = FindContent { content_key };
        self.discovery
            .send_talkreq(enr, Message::Request(Request::FindContent(msg)).to_bytes())
            .await
    }

    pub fn handle_request(&self, request: Request) -> Response {
        match request {
            Request::Ping(Ping {
                enr_seq,
                data_radius,
            }) => {
                let enr_seq = self.discovery.local_enr().seq();
                Response::Pong(Pong {
                    enr_seq: enr_seq as u32,
                    data_radius: self.data_radius,
                })
            }

            Request::FindNodes(FindNodes { distances }) => {
                let enrs = self.discovery.find_nodes_response(distances);
                Response::Nodes(Nodes {
                    total: enrs.len() as u8,
                    enrs,
                })
            }
            // TODO
            Request::FindContent(FindContent { content_key }) => {
                Response::FoundContent(FoundContent { enrs: vec![] })
            }
        }
    }
}
