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
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use std::cmp::min;

pub struct GeneratorChaCha20 {
    rng:    ChaCha20Rng,
    buf:    [u8; GeneratorChaCha20::OUTSIZE],
}

impl GeneratorChaCha20 {
    /// Size of the output data.
    pub const OUTSIZE: usize = 102400;
    /// Chunk size. Multiple of the generator output size.
    pub const CHUNKFACTOR: usize = 64;

    pub fn new(seed: &Vec<u8>) -> GeneratorChaCha20 {
        assert!(seed.len() > 0);
        let mut trunc_seed = [0u8; 32];
        let len = min(trunc_seed.len(), seed.len());
        trunc_seed[0..len].copy_from_slice(&seed[0..len]);

        let rng = ChaCha20Rng::from_seed(trunc_seed);
        let buf = [0; GeneratorChaCha20::OUTSIZE];

        GeneratorChaCha20 {
            rng,
            buf,
        }
    }
}

impl NextRandom for GeneratorChaCha20 {
    fn get_size(&self) -> usize {
        GeneratorChaCha20::OUTSIZE
    }

    fn next(&mut self) -> &[u8] {
        self.rng.fill_bytes(&mut self.buf);

        &self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmp_result() {
        let mut a = GeneratorChaCha20::new(&vec![1,2,3]);
        fn reduce(acc: u32, (i, x): (usize, &u8)) -> u32 {
            acc.rotate_left(i as u32) ^ (*x as u32)
        }
        assert_eq!(a.next().iter().enumerate().fold(0, reduce), 704022184);
        assert_eq!(a.next().iter().enumerate().fold(0, reduce), 1786387739);
        assert_eq!(a.next().iter().enumerate().fold(0, reduce), 3733544090);
        assert_eq!(a.next().iter().enumerate().fold(0, reduce), 3339470250);
    }

    #[test]
    fn test_seed_equal() {
        let mut a = GeneratorChaCha20::new(&vec![1,2,3]);
        let mut b = GeneratorChaCha20::new(&vec![1,2,3]);
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
        let mut a = GeneratorChaCha20::new(&vec![1,2,3]);
        let mut b = GeneratorChaCha20::new(&vec![1,2,4]);
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
