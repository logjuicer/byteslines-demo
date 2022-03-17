// Copyright (C) 2022 Red Hat
// SPDX-License-Identifier: Apache-2.0

use std::io::{BufRead, BufReader};

pub fn main() {
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    let reader = BufReader::new(handle);
    let mut total = 0;
    for line in reader.lines() {
        let line = line.unwrap();
        total += line.len();
    }
    println!("Total: {}", total)
}
