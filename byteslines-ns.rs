// Copyright (C) 2022 Red Hat
// SPDX-License-Identifier: Apache-2.0

use bytes::{Buf, Bytes, BytesMut};
use std::io::{Read, Result};

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    EoF,
    Scanning(usize),
}

/// The BytesLines struct holds a single buffer to store the read data and it yields immutable memory slice.
pub struct BytesLines<R: Read> {
    reader: R,
    buf: BytesMut,
    state: State,
}

/// Logline is a tuple (content, line number).

impl<R: Read> Iterator for BytesLines<R> {
    type Item = Result<Bytes>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            State::EoF => None,
            State::Scanning(_) if self.buf.is_empty() => self.read_slice(),
            State::Scanning(_) => self.get_slice(),
        }
    }
}

const CHUNK_SIZE: usize = 8192;

impl<R: Read> BytesLines<R> {
    /// Creates a new BytesLines.
    pub fn new(reader: R) -> BytesLines<R> {
        BytesLines {
            reader,
            state: State::Scanning(0),
            buf: BytesMut::with_capacity(CHUNK_SIZE),
        }
    }

    // Read a new chunk and call get_slice
    fn read_slice(&mut self) -> Option<Result<Bytes>> {
        let pos = self.buf.len();
        self.buf.resize(pos + CHUNK_SIZE, 0);
        match self.reader.read(&mut self.buf[pos..]) {
            // We read some data.
            Ok(n) if n > 0 => {
                self.buf.truncate(pos + n);
                self.get_slice()
            }

            // We reached the end of the reader, but we have left-overs.
            Ok(_) if pos > 0 => {
                self.state = State::EoF;
                Some(Ok(self.buf.split_to(pos).freeze()))
            }

            // We reached the end of the reader, this is the end.
            Ok(_) => None,

            // There was a reading error, we return it.
            Err(e) => Some(Err(e)),
        }
    }

    // Find the next line in the buffer
    fn get_slice(&mut self) -> Option<Result<Bytes>> {
        // TODO: keep track of last pos to skip the previous part
        match memchr::memchr('\n' as u8, self.buf.as_ref()) {
            // We haven't found the end of the line, we need more data.
            None => {
                // reserve() will attempt to reclaim space in the buffer.
                self.buf.reserve(CHUNK_SIZE);
                self.read_slice()
            }

            // We found the end of the line, we can return it now.
            Some(pos) => {
                // split_to() creates a new zero copy reference to the buffer.
                let res = self.buf.split_to(pos).freeze();
                self.buf.advance(1);
                Some(Ok(res))
            }
        }
    }
}

pub fn main() {
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    let mut total = 0;
    for line in BytesLines::new(handle) {
        let bytes = line.unwrap();
        let s = std::str::from_utf8(&bytes[..]).unwrap();
        total += s.len();
    }
    println!("Total: {}", total)
}
