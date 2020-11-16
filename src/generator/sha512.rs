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

use crate::generator::NextRandom;
use crate::generator::buffer::Buffer;
use crypto::{sha2::Sha512, digest::Digest};

pub struct GeneratorSHA512 {
    alg:        Sha512,
    buffer:     Buffer,
}

impl GeneratorSHA512 {
    /// Size of the base SHA512 algorithm, in bytes.
    const SIZE: usize = 512 / 8;
    /// Chunk size of previous hash to incorporate into the next hash.
    const PREVSIZE: usize = GeneratorSHA512::SIZE / 2;
    /// Size of the output data.
    pub const OUTSIZE: usize = GeneratorSHA512::SIZE;
    /// Chunk size. Multiple of the generator output size.
    pub const CHUNKFACTOR: usize = 1024 * 10;

    pub fn new(seed: &Vec<u8>) -> GeneratorSHA512 {
        GeneratorSHA512 {
            alg:        Sha512::new(),
            buffer:     Buffer::new(seed,
                                    GeneratorSHA512::SIZE,
                                    GeneratorSHA512::PREVSIZE),
        }
    }
}

impl NextRandom for GeneratorSHA512 {
    fn get_size(&self) -> usize {
        GeneratorSHA512::OUTSIZE
    }

    fn next(&mut self) -> &[u8] {
        // Increment the counter.
        self.buffer.next_count();

        // Calculate the next hash.
        self.alg.input(self.buffer.hashalg_input());

        // Get the hash and store it into the input buffer (for next iteration).
        self.alg.result(self.buffer.hashalg_output());
        self.alg.reset();

        // Return the generated hash.
        &self.buffer.get_result()[..GeneratorSHA512::OUTSIZE]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmp_result() {
        let mut a = GeneratorSHA512::new(&vec![1,2,3]);
        fn reduce(acc: u32, (i, x): (usize, &u8)) -> u32 {
            acc.rotate_left(i as u32) ^ (*x as u32)
        }
        assert_eq!(a.next().iter().enumerate().fold(0, reduce), 2314945247);
        assert_eq!(a.next().iter().enumerate().fold(0, reduce), 1602996934);
        assert_eq!(a.next().iter().enumerate().fold(0, reduce), 3995525905);
        assert_eq!(a.next().iter().enumerate().fold(0, reduce), 2890628318);
    }

    #[test]
    fn test_seed_equal() {
        let mut a = GeneratorSHA512::new(&vec![1,2,3]);
        let mut b = GeneratorSHA512::new(&vec![1,2,3]);
        let mut res_a = vec![];
        let mut res_b = vec![];
        for _ in 0..2 {
            res_a.push(a.next().to_vec());
            res_b.push(b.next().to_vec());
        }
        assert_eq!(res_a[0], res_b[0]);
        assert_eq!(res_a[1], res_b[1]);
        assert_ne!(res_a[0], res_a[1]);
        assert_ne!(res_b[0], res_b[1]);
    }

    #[test]
    fn test_seed_diff() {
        let mut a = GeneratorSHA512::new(&vec![1,2,3]);
        let mut b = GeneratorSHA512::new(&vec![1,2,4]);
        let mut res_a = vec![];
        let mut res_b = vec![];
        for _ in 0..2 {
            res_a.push(a.next().to_vec());
            res_b.push(b.next().to_vec());
        }
        assert_ne!(res_a[0], res_b[0]);
        assert_ne!(res_a[1], res_b[1]);
        assert_ne!(res_a[0], res_a[1]);
        assert_ne!(res_b[0], res_b[1]);
    }
}

// vim: ts=4 sw=4 expandtab
