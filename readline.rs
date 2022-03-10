// Copyright (C) 2022 Red Hat
// SPDX-License-Identifier: Apache-2.0

use std::io::Read;
use std::io::Result;
use std::io::{BufRead, BufReader};

pub struct BufLines<R: Read> {
    reader: BufReader<R>,
    buffer: String,
    line_number: usize,
}

type LogLine = (String, usize);

impl<R: Read> Iterator for BufLines<R> {
    type Item = Result<LogLine>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            match self.reader.read_line(&mut self.buffer) {
                Ok(n) if n > 0 => {
                    self.buffer = self
                        .buffer
                        .trim_end_matches(|c| matches!(c, '\n' | '\r'))
                        .to_owned();
                    self.get_line()
                }
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            }
        } else {
            self.get_line()
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

    fn get_line(&mut self) -> Option<Result<LogLine>> {
        let line = if let Some((sub_line, rest)) = self.buffer.split_once("\\n") {
            let line = sub_line.to_owned();
            self.buffer = rest.to_owned();
            line
        } else {
            self.line_number += 1;
            // Here the buffer is copied to return a String
            let line = self.buffer.clone();
            self.buffer.clear();
            line
        };
        Some(Ok((line, self.line_number)))
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
