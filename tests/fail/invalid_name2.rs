/*
 * Copyright (c) 2025 Tomi Leppänen
 * SPDX-License-Identifier: MIT
 */

use sine_macro::sine_wave;

fn main() {
    let _wave = sine_wave!(invalid: 10, rate: 10000);
}
