use std::collections::HashMap;
use std::io::ErrorKind::WouldBlock;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;

use netlib::net::tcp::TcpStream;
use netlib::{Interest, PollReactor, Reaction, Reactor, Result};

use common::Message;

use crate::connections::Connections;
use crate::DefaultCodec;

fn authenticate(username: &str, password: &str) -> bool {
    username == "florp" && password == "test"
}

// -----------------------------------------------------------------------------
//     - Sign In reactor -
// -----------------------------------------------------------------------------
pub struct SignIn<T: AsRawFd + Read + Write>(Connections<T, DefaultCodec, ()>);

impl<T: AsRawFd + Read + Write> SignIn<T> {
    pub(super) fn new() -> Self {
        Self(Connections::new())
    }
}

impl<T: AsRawFd + Read + Write> Reactor for SignIn<T> {
    type Input = PollReactor<T>;
    type Output = Self::Input;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(ev) if self.0.contains_key(&ev.owner) => {
                let mut data = self.0.recv(ev.owner);
                if let Some(mut data) = data.first_mut() {
                    let message = Message::from_bytes(&data);
                    if let Message::SignInRequest(username, password) = message {
                        if authenticate(&username, &password) {
                            self.0.send(ev.owner, Message::Hello(1).to_bytes());
                        } else {
                            self.0.send(ev.owner, Message::Hello(0).to_bytes());
                        }
                    }
                }
                Reaction::Continue
            }

            // -----------------------------------------------------------------------------
            //     - Incoming connections -
            // -----------------------------------------------------------------------------
            Reaction::Value(val) => {
                self.0.insert(val.id, (val, DefaultCodec::new()));
                Reaction::Continue
            }

            Reaction::Event(ev) => Reaction::Event(ev),
            Reaction::Continue => Reaction::Continue,
        }
    }
}
