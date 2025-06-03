/*
 * Copyright (c) 2025 Tomi Leppänen
 * SPDX-License-Identifier: MIT
 */

//! Just to check that the crate is usable in no_std contexts

#![no_std]

use sine_macro::sine_wave;

#[test]
fn test_nostd() {
    const WAVE: [i8; 10] = [0, 74, 120, 120, 74, 0, -74, -120, -120, -74];
    let wave = sine_wave!(frequency: 10, rate: 100, type: i8);
    assert_eq!(wave, WAVE);
}
