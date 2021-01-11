use std::ops::Range;
use std::convert::TryInto;
use std::mem::size_of;
use std::io::{Read, Write, self};
use std::marker::PhantomData;

use serde::de::DeserializeOwned;

use crate::codec::Codec;

const BUF_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Header {
    Unset,
    Small, // Content length is  u8::MAX
    Large, // Content length is u32::MAX
}

impl Header {
    fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Header::Unset),
            1 => Some(Header::Small),
            2 => Some(Header::Large),
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct Frame {
    buffer: Vec<u8>,
    header: Header,
    size: usize,
    bytes_read: usize,
}

impl Frame {
    pub fn empty() -> Self {
        let mut buffer = Vec::with_capacity(BUF_SIZE);
        unsafe { buffer.set_len(BUF_SIZE) };

        Self {
            buffer,
            header: Header::Unset,
            bytes_read: 0,
            size: 0,
        }
    }

    pub fn read<T: Read>(&mut self, reader: &mut T) -> io::Result<usize> {
        let res = reader.read(&mut self.buffer);
        if let Ok(n) = res {
            self.bytes_read += n
        }
        res
    }

    pub fn try_frame<T: DeserializeOwned>(&mut self) -> Option<T> {
        let offset = match self.header {
            Header::Small => 2,
            Header::Large => 5,
            Header::Unset => return None,
        };

        if self.size == 0 {
            return None;
        }

        // Make sure we have enough data to deserialize
        if self.bytes_read < self.size + offset {
            return None;
        }

        let range = offset..self.size + offset;
        let res = serde_json::from_slice(&self.buffer[range.clone()]);

        self.shift(range);

        if self.bytes_read <= BUF_SIZE && self.buffer.capacity() > BUF_SIZE {
            unsafe { self.buffer.set_len(BUF_SIZE) };
            self.buffer.shrink_to_fit();
        }

        res.ok()
    }

    fn shift(&mut self, Range { start, end }: Range<usize>) {
        unsafe { 
            let src = self.buffer.as_ptr().add(end);
            let dst = self.buffer.as_mut_ptr().add(start);
            std::ptr::copy(src, dst, self.bytes_read);
            self.bytes_read -= end - start;
        }
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Serialize, Deserialize)] 
    struct Message {
        name: String,
    }

    impl Message {
        fn new() -> Self {
            Self {
                name: "Lark".into(),
            }
        }
    }

    #[test]
    fn frame_message() {
        let message = Message::new();
        let message_data: Vec<u8> = serde_json::to_vec(&message).unwrap();
        let frame = Frame::from_message(message_data);
    }

    #[test]
    fn auto_grow_buffer() {
    }
}
