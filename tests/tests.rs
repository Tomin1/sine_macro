/*
 * Copyright (c) 2025 Tomi Leppänen
 * SPDX-License-Identifier: MIT
 */

use sine_macro::sine_wave;

const WAVE_100_10: [i16; 10] = [
    0, 19259, 31163, 31163, 19259, 0, -19259, -31163, -31163, -19259,
];

sine_wave! {
    static STATIC_WAVE = sine_wave(frequency: 10, rate: 100);
}

sine_wave! {
    static mut STATIC_MUT_WAVE = sine_wave(frequency: 10, rate: 100);
}

sine_wave! {
    const CONST_WAVE = sine_wave(frequency: 10, rate: 100);
}

sine_wave! {
    pub const EXPORTED_WAVE = sine_wave(frequency: 100);
}

sine_wave! {
    pub static EXPORTED_WAVE2 = sine_wave(frequency: 100);
}

sine_wave! {
    pub static mut EXPORTED_WAVE3 = sine_wave(frequency: 100);
}

#[test]
fn test_44100_441() {
    const WAVE_44100_441: [i16; 100] = [
        0, 2057, 4106, 6139, 8148, 10125, 12062, 13951, 15785, 17557, 19259, 20886, 22430, 23886,
        25247, 26509, 27666, 28713, 29648, 30465, 31163, 31737, 32186, 32508, 32702, 32767, 32702,
        32508, 32186, 31737, 31163, 30465, 29648, 28713, 27666, 26509, 25247, 23886, 22430, 20886,
        19259, 17557, 15785, 13951, 12062, 10125, 8148, 6139, 4106, 2057, 0, -2057, -4106, -6139,
        -8148, -10125, -12062, -13951, -15785, -17557, -19259, -20886, -22430, -23886, -25247,
        -26509, -27666, -28713, -29648, -30465, -31163, -31737, -32186, -32508, -32702, -32767,
        -32702, -32508, -32186, -31737, -31163, -30465, -29648, -28713, -27666, -26509, -25247,
        -23886, -22430, -20886, -19259, -17557, -15785, -13951, -12062, -10125, -8148, -6139,
        -4106, -2057,
    ];
    let wave = sine_wave!(frequency: 441, rate: 44100);
    assert_eq!(wave, WAVE_44100_441);
    let wave = sine_wave!(rate: 44100, frequency: 441);
    assert_eq!(wave, WAVE_44100_441);
    let wave = sine_wave!(frequency: 441);
    assert_eq!(wave, WAVE_44100_441);
}

#[test]
fn test_44100_441_partial() {
    const WAVE_44100_441: [i16; 10] =
        [0, 2057, 4106, 6139, 8148, 10125, 12062, 13951, 15785, 17557];
    let wave = sine_wave!(frequency: 441, rate: 44100, len: 10);
    assert_eq!(wave, WAVE_44100_441);
}

#[test]
fn test_100_10_static() {
    assert_eq!(STATIC_WAVE, WAVE_100_10);
}

#[test]
fn test_100_10_static_mut() {
    assert_eq!(unsafe { STATIC_MUT_WAVE }, WAVE_100_10);
}

#[test]
fn test_100_10_const() {
    assert_eq!(CONST_WAVE, WAVE_100_10);
}

#[test]
fn test_100_10_repeated() {
    const WAVE_100_10: [i16; 20] = [
        0, 19259, 31163, 31163, 19259, 0, -19259, -31163, -31163, -19259, 0, 19259, 31163, 31163,
        19259, 0, -19259, -31163, -31163, -19259,
    ];
    let wave = sine_wave!(frequency: 10, rate: 100, repeats: 2);
    assert_eq!(wave, WAVE_100_10);
    let wave = sine_wave!(frequency: 10, rate: 100, len: 20);
    assert_eq!(wave, WAVE_100_10);
}

#[test]
fn test_100_10_shifted() {
    const WAVE_100_10: [i16; 10] = [
        0, -19259, -31163, -31163, -19259, 0, 19259, 31163, 31163, 19259,
    ];
    let wave = sine_wave!(frequency: 10, rate: 100, skip: 5);
    assert_eq!(wave, WAVE_100_10);
    let wave = sine_wave!(frequency: 10, rate: 100, skip: 15);
    assert_eq!(wave, WAVE_100_10);
}

#[test]
fn test_100_10_i8() {
    const WAVE_100_10: [i8; 10] = [0, 74, 120, 120, 74, 0, -74, -120, -120, -74];
    sine_wave! {
        const WAVE = sine_wave(frequency: 10, rate: 100, type: i8);
    }
    assert_eq!(WAVE, WAVE_100_10);
}

#[test]
fn test_100_10_i16() {
    const WAVE_100_10: [i16; 10] = [
        0, 19259, 31163, 31163, 19259, 0, -19259, -31163, -31163, -19259,
    ];
    sine_wave! {
        const WAVE = sine_wave(frequency: 10, rate: 100, type: i16);
    }
    assert_eq!(WAVE, WAVE_100_10);
}

#[test]
fn test_100_10_i32() {
    const WAVE_100_10: [i32; 10] = [
        0,
        1262259217,
        2042378316,
        2042378316,
        1262259217,
        0,
        -1262259217,
        -2042378316,
        -2042378316,
        -1262259217,
    ];
    sine_wave! {
        const WAVE = sine_wave(frequency: 10, rate: 100, type: i32);
    }
    assert_eq!(WAVE, WAVE_100_10);
}
