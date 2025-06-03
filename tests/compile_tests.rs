/*
 * Copyright (c) 2025 Tomi Leppänen
 * SPDX-License-Identifier: MIT
 */

#[test]
fn test_compile_no_arguments() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/no_arguments.rs");
}

#[test]
fn test_compile_bad_name() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/bad_name_const.rs");
    t.compile_fail("tests/fail/bad_name_static.rs");
}

#[test]
fn test_compile_zero_values() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/zero_frequency.rs");
    t.compile_fail("tests/fail/zero_rate.rs");
    t.compile_fail("tests/fail/zero_len.rs");
    t.compile_fail("tests/fail/zero_repeats.rs");
}

#[test]
fn test_compile_negative_values() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/negative_frequency.rs");
    t.compile_fail("tests/fail/negative_rate.rs");
    t.compile_fail("tests/fail/negative_len.rs");
    t.compile_fail("tests/fail/negative_repeats.rs");
    t.compile_fail("tests/fail/negative_skip.rs");
}

#[test]
fn test_compile_invalid_name() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/invalid_name.rs");
    t.compile_fail("tests/fail/invalid_name2.rs");
    t.compile_fail("tests/fail/invalid_name3.rs");
    t.compile_fail("tests/fail/invalid_name4.rs");
}

#[test]
fn test_compile_invalid_values() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/invalid_frequency.rs");
    t.compile_fail("tests/fail/invalid_rate.rs");
    t.compile_fail("tests/fail/invalid_type.rs");
    t.compile_fail("tests/fail/invalid_values.rs");
    t.compile_fail("tests/fail/invalid_values2.rs");
    t.compile_fail("tests/fail/invalid_values3.rs");
}

#[test]
fn test_compile_defined_twice() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/twice_frequency.rs");
    t.compile_fail("tests/fail/twice_rate.rs");
    t.compile_fail("tests/fail/twice_len.rs");
    t.compile_fail("tests/fail/twice_repeats.rs");
    t.compile_fail("tests/fail/twice_skip.rs");
    t.compile_fail("tests/fail/twice_type.rs");
}

#[test]
fn test_compile_both_repeats_and_len() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/len_and_repeats.rs");
    t.compile_fail("tests/fail/repeats_and_len.rs");
}
