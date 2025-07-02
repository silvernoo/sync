use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub enum ClipboardData {
    Text(String),
    Image {
        width: u32,
        height: u32,
        bytes: Vec<u8>,
    },
}
