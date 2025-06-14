/*
 * Copyright (c) 2025 Tomi Leppänen
 * SPDX-License-Identifier: MIT
 */

use sine_macro::sine_wave;

fn main() {
    let _wave = sine_wave!(rate: 2000, len: 100, skip: 50, invalid: 123);
}
