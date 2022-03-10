// Copyright (C) 2022 Red Hat
// SPDX-License-Identifier: Apache-2.0

use bytes::{Buf, Bytes, BytesMut};
use std::io::{Read, Result};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Sep {
    // A line return: '\n'
    NewLine,
    // A litteral line return: '\\n'
    SubLine,
}

impl Sep {
    // The size of the separator
    fn len(&self) -> usize {
        match self {
            Sep::NewLine => 1,
            Sep::SubLine => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    // We reached the end of the file.
    EoF,
    // We are processing a line, keeping track of the last separator to properly increase the line count.
    Scanning(Sep),
}

/// The BytesLines struct holds a single buffer to store the read data and it yields immutable memory slice.
pub struct BytesLines<R: Read> {
    reader: R,
    buf: BytesMut,
    state: State,
    line_count: usize,
}

/// Logline is a tuple (content, line number).
pub type LogLine = (Bytes, usize);

impl<R: Read> Iterator for BytesLines<R> {
    type Item = Result<LogLine>;

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
            state: State::Scanning(Sep::NewLine),
            buf: BytesMut::with_capacity(CHUNK_SIZE),
            line_count: 0,
        }
    }

    // Read a new chunk and call get_slice
    fn read_slice(&mut self) -> Option<Result<LogLine>> {
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
                self.update_line_counter(State::EoF);
                Some(Ok((self.buf.split_to(pos).freeze(), self.line_count)))
            }

            // We reached the end of the reader, this is the end.
            Ok(_) => None,

            // There was a reading error, we return it.
            Err(e) => Some(Err(e)),
        }
    }

    // Find the next line in the buffer
    fn get_slice(&mut self) -> Option<Result<LogLine>> {
        match self.find_next_line() {
            // We haven't found the end of the line, we need more data.
            None => {
                // reserve() will attempt to reclaim space in the buffer.
                self.buf.reserve(CHUNK_SIZE);
                self.read_slice()
            }

            // We found the end of the line, we can return it now.
            Some((pos, t)) => {
                // split_to() creates a new zero copy reference to the buffer.
                let res = self.buf.split_to(pos).freeze();
                self.buf.advance(t.len());
                Some(Ok((res, self.line_count)))
            }
        }
    }

    // Find the next line position and update the line count
    fn find_next_line(&mut self) -> Option<(usize, Sep)> {
        let slice = self.buf.as_ref();
        let size = slice.len();
        let char_is = |pos: usize, c: char| pos < size && slice[pos] == (c as u8);
        for pos in 0..size {
            let c: char = slice[pos] as char;
            let sep = match c {
                '\n' => Some(Sep::NewLine),
                '\\' if char_is(pos + 1, 'n') => Some(Sep::SubLine),
                _ => None,
            };
            if let Some(sep) = sep {
                // We found a separator.
                self.update_line_counter(State::Scanning(sep));
                return Some((pos, sep));
            }
        }
        None
    }

    fn update_line_counter(&mut self, state: State) {
        // We only increase the line counter when the last separator was a new line.
        if self.state == State::Scanning(Sep::NewLine) {
            self.line_count += 1
        }
        self.state = state;
    }
}

pub fn main() {
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    let mut total = 0;
    for line in BytesLines::new(handle) {
        let (bytes, _) = line.unwrap();
        let s = std::str::from_utf8(&bytes[..]).unwrap();
        total += s.len();
    }
    println!("Total: {}", total)
}
