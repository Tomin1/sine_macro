Procedural rust macro for generating sine wave arrays
=====================================================
This provides a procedural macro for generating signed integer sine waves as
arrays.

[![Crates.io][cratesio-version]][cratesio-link]
[![MIT licensed][cratesio-license]](LICENSE)
[![docs.rs][docsrs-badge]][docsrs-link]

[cratesio-version]: https://img.shields.io/crates/v/sine_macro
[cratesio-license]: https://img.shields.io/crates/l/sine_macro
[cratesio-link]: https://crates.io/crates/sine_macro
[docsrs-badge]: https://img.shields.io/docsrs/sine_macro
[docsrs-link]: https://docs.rs/sine_macro/latest/sine_macro/

Example
-------
```rust
use sine_macro::sine_wave;

// Sine wave defined as const:
sine_wave! {
    const MY_CONST_SINE_WAVE = sine_wave(frequency: 440, rate: 48_000);
}

// Sine wave defined as local variable with default rate of 44,100 Hz:
let wave = sine_wave!(frequency: 440);
```

For more examples, please check [the documentation][docsrs-link].

License
-------
This crate is MIT licensed. See [LICENSE](LICENSE) for more information.
Dependency crates have their own licenses.
