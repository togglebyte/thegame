use std::sync::mpsc::{self, Sender, Receiver};

use tinybit::{WorldPos, Pixel};
use serde::{Deserialize, Serialize};

pub type Rx = Receiver<Message>;
pub type Tx = Sender<Message>;

pub fn channel() -> (Tx, Rx) {
    mpsc::channel()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveState {
    pub tilemap: Vec<Pixel>,
    pub player_pos: WorldPos,
    pub player_id: usize,
    pub inventory: Vec<()>,
}


#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    SignInRequest(String, String),
    SignInResponse(SaveState),
    SignInSuccess(bool),
}

impl Message {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = serde_json::to_vec(&self).unwrap();
        bytes.push(b'\n');
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        serde_json::from_slice(&bytes[..bytes.len()]).unwrap()
    }
}
