Procedural rust macro for generating sine wave arrays
=====================================================
This provides a procedural macro for generating signed integer sine waves as
arrays. Mainly useful for producing different beep sounds on embedded systems
which might not even have `alloc` crate or a floating point unit.

[![Crates.io][cratesio-version]][cratesio-link]
[![MIT licensed][cratesio-license]](LICENSE)
[![docs.rs][docsrs-badge]][docsrs-link]

[cratesio-version]: https://img.shields.io/crates/v/sine_macro
[cratesio-license]: https://img.shields.io/crates/l/sine_macro
[cratesio-link]: https://crates.io/crates/sine_macro
[docsrs-badge]: https://img.shields.io/docsrs/sine_macro
[docsrs-link]: https://docs.rs/sine_macro/latest/sine_macro/

Usage
-----
Add as a dependency:
```sh
cargo add sine_macro
```

Amend your code:
```rust
use sine_macro::sine_wave;

// Sine wave defined as const:
sine_wave! {
    const MY_CONST_SINE_WAVE = sine_wave(frequency: 440, rate: 48_000);
}

// Or define sine wave as a local variable:
let wave = sine_wave!(frequency: 440, rate: 48_000);
```

These are both arrays of type `[i16; 109]`. Some rounding will be applied when
the sampling rate is not an exact multiple of the frequency.

For more knobs and examples, please see [the documentation][docsrs-link].

License
-------
This crate is MIT licensed. See [LICENSE](LICENSE) for more information.
Dependency crates have their own licenses.
