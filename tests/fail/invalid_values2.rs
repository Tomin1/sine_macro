/*
 * Copyright (c) 2025 Tomi Leppänen
 * SPDX-License-Identifier: MIT
 */

use sine_macro::sine_wave;

fn main() {
    let _wave = sine_wave!(rate: 3000, frequency: 4000);
}
