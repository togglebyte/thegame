use serde::{Deserialize, Serialize};

use super::GameState;

#[derive(Debug, Serialize, Deserialize)]
pub enum Auth {
    Failed,
    Success(GameState),
    SignIn(String, String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Auth(Auth),
}

impl Message {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = serde_json::to_vec(&self).unwrap();
        bytes.push(b'\n');
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        if let Ok(s) = std::str::from_utf8(bytes) {
            eprintln!("{:?}", s);
        }
        serde_json::from_slice(bytes).unwrap()
    }
}
