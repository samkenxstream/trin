use ssz::DecodeError;
use uint::construct_uint;

use crate::discovery::Enr;
use rlp::{Decodable, Encodable};
use ssz_derive::{Decode, Encode};

construct_uint! {
    /// 256-bit unsigned integer.
    pub(super) struct U256(4);
}

// Taken from https://github.com/sigp/lighthouse/blob/stable/consensus/ssz/src/encode/impls.rs
impl ssz::Encode for U256 {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        32
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        let n = <Self as ssz::Encode>::ssz_fixed_len();
        let s = buf.len();

        buf.resize(s + n, 0);
        self.to_little_endian(&mut buf[s..]);
    }
}

// Taken from https://github.com/sigp/lighthouse/blob/stable/consensus/ssz/src/decode/impls.rs
impl ssz::Decode for U256 {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        32
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let len = bytes.len();
        let expected = <Self as ssz::Decode>::ssz_fixed_len();

        if len != expected {
            Err(DecodeError::InvalidByteLength { len, expected })
        } else {
            Ok(U256::from_little_endian(bytes))
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ProtocolMessage {
    message_id: u8,
    encoded_message: Message,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    Request(Request),
    Response(Response),
}

// impl Message {
//     pub fn to_bytes(&self) -> Vec<u8> {
//         match self {
//             Message::Request(r) =>
//         }
//     }
// }

#[derive(Debug, PartialEq, Clone)]
pub enum Request {
    Ping(Ping),
    FindNodes(FindNodes),
    FindContent(FindContent),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Response {
    Pong(Pong),
    Nodes(Nodes),
    FoundContent(FoundContent),
}

#[derive(Debug, PartialEq, Clone, Encode, Decode)]
pub struct Ping {
    enr_seq: u32,
    data_radius: U256,
}

#[derive(Debug, PartialEq, Clone, Encode, Decode)]
pub struct Pong {
    enr_seq: u32,
    data_radius: U256,
}

#[derive(Debug, PartialEq, Clone, Encode, Decode)]
pub struct FindNodes {
    // TODO: Make this an ssz list
    distances: Vec<u16>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Nodes {
    total: u8,
    // TODO: Make this an ssz list
    enrs: Vec<Enr>,
}

// TODO: check correctness and if there's a better way
impl ssz::Encode for Nodes {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.push(self.total);
        for enr in self.enrs {
            buf.append(enr.rlp_bytes().to_vec().as_mut());
        }
    }
}

// TODO: check correctness and if there's a better way
impl ssz::Decode for Nodes {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() == 0 {
            return Err(DecodeError::BytesInvalid("Should not be empty".to_string()));
        }
        let total = bytes.first().expect("should have one element");
        let enr_bytes = <Vec<Vec<u8>>>::from_ssz_bytes(&bytes[1..])?;
        let enrs: Result<Vec<Enr>, _> = enr_bytes
            .into_iter()
            .map(|bytes| {
                rlp::decode(&bytes)
                    .map_err(|e| DecodeError::BytesInvalid(format!("rlp decoding failed: {}", e)))
            })
            .collect();
        Ok(Self {
            total: *total,
            enrs: enrs?,
        })
    }
}

#[derive(Debug, PartialEq, Clone, Encode, Decode)]
pub struct FindContent {
    // TODO: Use some version of H256
    content_key: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone, Encode, Decode)]
pub struct FoundContent {
    enrs: Vec<Enr>,
    payload: Vec<u8>,
}
