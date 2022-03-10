// Copyright (C) 2022 Red Hat
// SPDX-License-Identifier: Apache-2.0

use std::io::Read;
use std::io::Result;
use std::io::{BufRead, BufReader};

/// A struct to hold the state of the iterator.
pub struct BufLines<R: Read> {
    reader: BufReader<R>,
    buffer: String,
    line_number: usize,
}

/// The Iterator implementation.
impl<R: Read> Iterator for BufLines<R> {
    type Item = Result<(String, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            self.read_line()
        } else {
            Some(Ok(self.get_line()))
        }
    }
}

impl<R: Read> BufLines<R> {
    pub fn new(read: R) -> BufLines<R> {
        BufLines {
            reader: BufReader::new(read),
            buffer: String::new(),
            line_number: 0,
        }
    }

    fn read_line(&mut self) -> Option<Result<(String, usize)>> {
        match self.reader.read_line(&mut self.buffer) {
            Ok(n) if n > 0 => {
                // The read succeeded
                self.buffer = self.buffer.trim_end().to_owned();
                Some(Ok(self.get_line()))
            }
            Ok(_) => None,
            Err(e) => Some(Err(e)),
        }
    }

    // Return the first sub line found in the buffer.
    fn get_line(&mut self) -> (String, usize) {
        let line = if let Some((sub_line, rest)) = self.buffer.split_once("\\n") {
            let sub_line = sub_line.to_owned();
            self.buffer = rest.to_owned();
            sub_line
        } else {
            self.line_number += 1;
            let line = self.buffer.clone();
            self.buffer.clear();
            line
        };
        (line, self.line_number)
    }
}

pub fn main() {
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    let mut total = 0;
    for line in BufLines::new(handle) {
        let (line, _) = line.unwrap();
        total += line.len();
    }
    println!("Total: {}", total)
}
