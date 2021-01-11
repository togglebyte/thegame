use std::sync::mpsc;
use std::io::ErrorKind::WouldBlock;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use common::{Tx, Rx};
use common::models::Message;

pub fn connect(client_tx: Tx) -> Tx {
    let (server_tx, server_rx) = mpsc::channel::<Message>();

    let mut connection = TcpStream::connect("127.0.0.1:9000").unwrap();
    connection.set_nonblocking(true);

    let handle = thread::spawn(move || {
        let mut buf = [0u8; 1024 * 8];

        loop {
            // -----------------------------------------------------------------------------
            //     - Receiving messages -
            // -----------------------------------------------------------------------------
            match connection.read(&mut buf) {
                Ok(0) => {}
                Ok(n) => {
                    let message = &buf[..n];
                    let message = Message::from_bytes(message);
                    client_tx.send(message);
                }
                Err(ref e) if e.kind() == WouldBlock => {}
                Err(e) => {
                    eprintln!("connection error: {:?}", e);
                    break;
                }
            }
            
            // -----------------------------------------------------------------------------
            //     - Sending messages -
            // -----------------------------------------------------------------------------
            match server_rx.try_recv() {
                Ok(mut message) => {
                    let _ = connection.write(&message.to_bytes());
                }
                Err(_) => {
                }
            }

            thread::sleep(Duration::from_millis(20));
        }
    });

    server_tx
}
