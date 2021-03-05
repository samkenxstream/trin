use discv5::enr::{CombinedKey, EnrBuilder, NodeId};
use discv5::{Discv5, Discv5Config};
use std::net::{IpAddr, SocketAddr};

type Enr = discv5::enr::Enr<CombinedKey>;

pub const PROTOCOL: &str = "state-network";

#[derive(Clone)]
pub struct Config {
    listen_address: IpAddr,
    listen_port: u16,
    discv5_config: Discv5Config,
    bootnode_enrs: Vec<Enr>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0".parse().expect("valid ip address"),
            listen_port: 4242,
            discv5_config: Discv5Config::default(),
            bootnode_enrs: vec![],
        }
    }
}

pub type ProtocolRequest = Vec<u8>;

pub struct Discovery {
    discv5: Discv5,
}

impl Discovery {
    pub async fn new(config: Config, protocol_callback: ) -> Result<Self, String> {
        let enr_key = CombinedKey::generate_secp256k1();

        let listen_socket = SocketAddr::new(config.listen_address, config.listen_port);

        let enr = {
            let mut builder = EnrBuilder::new("v4");
            builder.ip(config.listen_address);
            builder.udp(config.listen_port);
            builder.build(&enr_key).unwrap()
        };
        let mut discv5 = Discv5::new(enr, enr_key, config.discv5_config)
            .map_err(|e| format!("Failed to create discv5 instance: {}", e))?;

        for enr in config.bootnode_enrs {
            discv5
                .add_enr(enr)
                .map_err(|e| format!("Failed to add enr: {}", e))?;
        }
        discv5
            .start(listen_socket)
            .await
            .map_err(|e| format!("Failed to start discv5 server {:?}", e))?;
        Ok(Self { discv5 })
    }

    pub async fn discover_nodes(&mut self) -> Result<Vec<Enr>, String> {
        let random_node = NodeId::random();
        let nodes = self
            .discv5
            .find_node(random_node)
            .await
            .map_err(|e| format!("FindNode query failed: {:?}", e))?;
        Ok(nodes)
    }

    pub async fn send_request(
        &mut self,
        enr: Enr,
        request: ProtocolRequest,
    ) -> Result<Vec<u8>, String> {
        let response = self
            .discv5
            .talk_req(enr, PROTOCOL.as_bytes().to_vec(), request)
            .await
            .map_err(|e| format!("TalkReq query failed: {:?}", e))?;
        Ok(response)
    }
}
