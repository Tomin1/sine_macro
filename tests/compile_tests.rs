/*
 * Copyright (c) 2025 Tomi Leppänen
 * SPDX-License-Identifier: MIT
 */

#[test]
fn test_compile_bad_name() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/bad_name_const.rs");
    t.compile_fail("tests/fail/bad_name_static.rs");
}

#[test]
fn test_compile_zero_frequency() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/zero_frequency.rs");
}

#[test]
fn test_compile_zero_rate() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/zero_rate.rs");
}

#[test]
fn test_compile_negative_frequency() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/negative_frequency.rs");
}

#[test]
fn test_compile_negative_rate() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/negative_rate.rs");
}

#[test]
fn test_compile_invalid_frequency() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/invalid_frequency.rs");
}

#[test]
fn test_compile_invalid_rate() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/invalid_rate.rs");
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
    t.compile_fail("tests/fail/invalid_values.rs");
    t.compile_fail("tests/fail/invalid_values2.rs");
}

#[test]
fn test_compile_twice_frequency() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/twice_frequency.rs");
}

#[test]
fn test_compile_twice_rate() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/twice_rate.rs");
}

#[test]
fn test_compile_twice_len() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/twice_len.rs");
}
