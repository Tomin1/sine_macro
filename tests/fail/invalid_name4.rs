/*
 * Copyright (c) 2025 Tomi Leppänen
 * SPDX-License-Identifier: MIT
 */

use sine_macro::sine_wave;

fn main() {
    let _wave = sine_wave!(rate: 1000, frequency: 100, len: 1000, skip: 0, invalid: 10);
}
