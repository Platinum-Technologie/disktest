// -*- coding: utf-8 -*-
//
// disktest - Hard drive tester
//
// Copyright 2020 Michael Buesch <m@bues.ch>
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//

mod disktest;
mod error;
mod hasher;
mod kdf;
mod stream;
mod stream_aggregator;
mod util;

use clap;
use crate::error::Error;
use crate::util::parsebytes;
use disktest::{Disktest, DtStreamType};
use std::fs::OpenOptions;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = clap::App::new("disktest")
        .about("Hard Disk (HDD), Solid State Disk (SSD), USB Stick, Memory Card (e.g. SD-Card) tester.\n\n\
This program can write a pseudo random stream to a disk, read it back \
and verify it by comparing it to the expected stream.")
        .arg(clap::Arg::with_name("device")
             .index(1)
             .required(true)
             .help("Device file of the disk."))
        .arg(clap::Arg::with_name("write")
             .long("write")
             .short("w")
             .help("Write pseudo random data to the device. \
If this option is not given, then disktest will operate in verify-mode instead. \
In verify-mode the disk will be read and compared to the expected pseudo random sequence."))
        .arg(clap::Arg::with_name("seek")
             .long("seek")
             .short("s")
             .takes_value(true)
             .help("Seek to the specified byte position on disk \
before starting the write/verify operation. This skips the specified \
amount of bytes."))
        .arg(clap::Arg::with_name("bytes")
             .long("bytes")
             .short("b")
             .takes_value(true)
             .help("Number of bytes to write/verify. \
If not given, then the whole disk will be overwritten/verified."))
        .arg(clap::Arg::with_name("algorithm")
             .long("algorithm")
             .short("A")
             .takes_value(true)
             .help("Select the hashing algorithm. \
The selection can be: SHA512 or CRC. Default: SHA512. \
Please note that CRC is *not* cryptographically strong! \
But CRC is very fast. Only choose CRC, if cryptographic strength is not required. \
If in doubt, use SHA512."))
        .arg(clap::Arg::with_name("seed")
             .long("seed")
             .short("S")
             .takes_value(true)
             .help("The seed to use for hash stream generation. \
The generated pseudo random sequence is cryptographically reasonably strong. \
If you want a unique pattern to be written to disk, supply a random seed to this parameter. \
If not given, then the pseudo random sequence will be the same for everybody and \
it will therefore not be secret.
The seed may be any random string (e.g. a long passphrase)."))
        .arg(clap::Arg::with_name("threads")
             .long("threads")
             .short("j")
             .takes_value(true)
             .help("The number of CPUs to use. \
The special value 0 will select the maximum number of online CPUs in the system. \
If the number of threads is equal to number of CPUs it is optimal for performance. \
This parameter must be equal during corresponding verify and --write mode runs. \
Otherwise the verification will fail. Default: 1"))
        .arg(clap::Arg::with_name("quiet")
             .long("quiet")
             .short("q")
             .takes_value(true)
             .help("Quiet level: 0: Normal verboseness (default). \
1: Reduced verboseness. \
2: No informational output."))
        .get_matches();

    let device = args.value_of("device").unwrap();
    let write = args.is_present("write");
    let seek = match parsebytes(args.value_of("seek").unwrap_or("0")) {
        Ok(x) => x,
        Err(e) => return Err(Box::new(Error::new(&format!("Invalid --seek value: {}", e)))),
    };
    let max_bytes = match parsebytes(args.value_of("bytes").unwrap_or(&u64::MAX.to_string())) {
        Ok(x) => x,
        Err(e) => return Err(Box::new(Error::new(&format!("Invalid --bytes value: {}", e)))),
    };
    let algorithm = match args.value_of("algorithm").unwrap_or("SHA512").to_uppercase().as_str() {
        "SHA512" => DtStreamType::SHA512,
        "CRC" => DtStreamType::CRC,
        x => return Err(Box::new(Error::new(&format!("Invalid --algorithm value: {}", x)))),
    };
    let seed = args.value_of("seed").unwrap_or("42");
    let threads: usize = match args.value_of("threads").unwrap_or("1").parse() {
        Ok(x) => {
            if x >= std::u16::MAX as usize + 1 {
                return Err(Box::new(Error::new(&format!("Invalid --threads value: Out of range"))))
            }
            x
        },
        Err(e) => return Err(Box::new(Error::new(&format!("Invalid --threads value: {}", e)))),
    };
    let quiet: u8 = match args.value_of("quiet").unwrap_or("0").parse() {
        Ok(x) => x,
        Err(e) => return Err(Box::new(Error::new(&format!("Invalid --quiet value: {}", e)))),
    };

    // Open the disk device.
    let path = Path::new(&device);
    let mut file = match OpenOptions::new().read(!write)
                                           .write(write)
                                           .create(write)
                                           .open(path) {
        Err(e) => {
            eprintln!("Failed to open file {:?}: {}", path, e);
            return Err(Box::new(e));
        },
        Ok(file) => file,
    };

    let seed = seed.as_bytes().to_vec();
    let mut disktest = match Disktest::new(algorithm, &seed, threads, &mut file, &path, quiet) {
        Ok(x) => x,
        Err(e) => {
            return Err(Box::new(e))
        },
    };
    if write {
        if let Err(e) = disktest.write(seek, max_bytes) {
            return Err(Box::new(e))
        }
    } else {
        if let Err(e) = disktest.verify(seek, max_bytes) {
            return Err(Box::new(e))
        }
    }
    return Ok(());
}

// vim: ts=4 sw=4 expandtab
