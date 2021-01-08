use std::collections::HashMap;
use std::io::ErrorKind::WouldBlock;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};
use std::os::unix::io::AsRawFd;

use netlib::{Event, PollReactor};

use crate::codec::{Codec, Decode, Encode};

// -----------------------------------------------------------------------------
//     - Connections -
// -----------------------------------------------------------------------------
pub struct Connections<T, U, D>
where
    T: AsRawFd + Read + Write,
    U: Codec,
{
    inner: HashMap<u64, (PollReactor<T>, U)>,
    data: HashMap<u64, D>,
}

// -----------------------------------------------------------------------------
//     - Deref -
// -----------------------------------------------------------------------------
impl<T, U, D> Deref for Connections<T, U, D>
where
    T: AsRawFd + Read + Write,
    U: Codec,
{
    type Target = HashMap<u64, (PollReactor<T>, U)>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, U, D> DerefMut for Connections<T, U, D>
where
    T: AsRawFd + Read + Write,
    U: Codec,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// -----------------------------------------------------------------------------
//     - Impl -
// -----------------------------------------------------------------------------
impl<T, U, D> Connections<T, U, D>
where
    T: AsRawFd + Read + Write,
    U: Codec,
{
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            data: HashMap::new(),
        }
    }

    pub fn update(&mut self, event: &Event) {
        self.inner
            .get_mut(&event.owner)
            .map(|(c, _)| c.update(event));
    }

    pub fn associate_data(&mut self, reactor_id: u64, data: D) {
        self.data.insert(reactor_id, data);
    }

    pub fn data(&mut self, reactor_id: u64) -> Option<&mut D> {
        self.data.get_mut(&reactor_id)
    }

    pub fn recv(&mut self, reactor_id: u64) -> Vec<U::Item> {
        let (mut con, mut codec) = match self.remove(&reactor_id) {
            Some((con, codec)) => (con, codec),
            None => return Vec::new(),
        };

        let mut data = Vec::new();

        loop {
            match codec.decode(&mut con) {
                Decode::Value(val) => data.push(val),
                Decode::NoValue => break,
                Decode::Failed => return data,
            }
        }

        self.insert(reactor_id, (con, codec));

        data
    }

    pub fn send(&mut self, reactor_id: u64, message: U::Item) -> Option<()> {
        let (mut con, mut codec) = self.remove(&reactor_id).unwrap();

        match codec.encode(&mut con, message) {
            Encode::Success => {
                self.insert(reactor_id, (con, codec));
                Some(())
            }
            Encode::Fail => {
                self.data.remove(&reactor_id);
                None
            }
        }
    }
}
