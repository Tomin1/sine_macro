/*
 * Copyright (c) 2025 Tomi Leppänen
 * SPDX-License-Identifier: MIT
 */

//! A procedural macro for generating signed integer sine waves as arrays.
//!
//! # Example
//! ```rust
//! use sine_macro::sine_wave;
//!
//! // Sine wave defined as const item:
//! sine_wave! {
//!     const MY_CONST_SINE_WAVE = sine_wave(frequency: 400, rate: 16_000);
//! }
//!
//! // Or as static item:
//! sine_wave! {
//!     static MY_STATIC_SINE_WAVE = sine_wave(frequency: 1000, rate: 48_000, len: 48_000);
//! }
//!
//! // Sine wave defined as local variable with default rate of 44,100 Hz:
//! let wave = sine_wave!(frequency: 800, repeats: 10);
//! ```
//!
//! See the macro documentation for [more examples][crate::sine_wave!#arguments-and-examples].

#![deny(missing_docs)]
#![forbid(unsafe_code)]

use itertools::Itertools;
use proc_macro2::{Delimiter, Group, Punct, Spacing, TokenStream, TokenTree};
use quote::quote;
use std::f64::consts::PI;
use std::iter::repeat_n;
use std::num::{NonZero, NonZeroU32, NonZeroUsize};
use syn::parse::{Error, Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{Ident, LitInt, Result, StaticMutability, Visibility, parse_macro_input};
use syn::{Token, parenthesized};

mod types;
use crate::types::helpers::{Ident as GetIdent, Literal as GetLiteral, Max as GetMax};
use crate::types::*;

const DEFAULT_RATE: u32 = 44_100;
const DEFAULT_TYPE: &str = "i16";

struct SineWaveAttrs {
    frequency: LitInt,
    rate: Option<LitInt>,
    len: Option<LitInt>,
    repeats: Option<LitInt>,
    skip: Option<LitInt>,
    ty: Option<Type>,
}

impl Parse for SineWaveAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = Punctuated::<AttrInput, Token![,]>::parse_terminated(input)?;
        let mut frequency = None;
        let mut rate: Option<LitInt> = None;
        let mut len = None;
        let mut repeats = None;
        let mut skip = None;
        let mut ty = None;
        for attr in attrs {
            match attr {
                AttrInput::Int(IntAttrInput {
                    name,
                    value: Int::Frequency(attr_value),
                    ..
                }) => {
                    if frequency.is_none() {
                        let value: NonZeroU32 = attr_value.base10_parse()?;
                        if let Some(rate) = &rate {
                            let rate: NonZeroU32 = rate.base10_parse().unwrap();
                            if rate < value {
                                return Err(Error::new_spanned(
                                    attr_value,
                                    format_args!(
                                        "`frequency` should be less than `rate`, which is {} Hz",
                                        rate
                                    ),
                                ));
                            }
                        }
                        frequency = Some(attr_value)
                    } else {
                        return Err(Error::new_spanned(name, "`frequency` defined twice"));
                    }
                }
                AttrInput::Int(IntAttrInput {
                    name,
                    value: Int::Rate(attr_value),
                    ..
                }) => {
                    if rate.is_none() {
                        let value: NonZeroU32 = attr_value.base10_parse()?;
                        if let Some(frequency) = &frequency {
                            let frequency: NonZeroU32 = frequency.base10_parse().unwrap();
                            if frequency > value {
                                return Err(Error::new_spanned(
                                    attr_value,
                                    format_args!(
                                        "`rate` should be more than `frequency`, which is {} Hz",
                                        frequency
                                    ),
                                ));
                            }
                        }
                        rate = Some(attr_value)
                    } else {
                        return Err(Error::new_spanned(name, "`rate` defined twice"));
                    }
                }
                AttrInput::Int(IntAttrInput {
                    name,
                    value: Int::Len(attr_value),
                    ..
                }) => {
                    if repeats.is_some() {
                        return Err(Error::new_spanned(
                            name,
                            "cannot define both `len` and `repeats`",
                        ));
                    } else if len.is_none() {
                        let _value: NonZeroUsize = attr_value.base10_parse()?;
                        len = Some(attr_value)
                    } else {
                        return Err(Error::new_spanned(name, "`len` defined twice"));
                    }
                }
                AttrInput::Int(IntAttrInput {
                    name,
                    value: Int::Repeats(attr_value),
                    ..
                }) => {
                    if len.is_some() {
                        return Err(Error::new(
                            name.span(),
                            "cannot define both `len` and `repeats`",
                        ));
                    } else if repeats.is_none() {
                        let value: usize = attr_value.base10_parse()?;
                        if value > 0 {
                            repeats = Some(attr_value)
                        } else {
                            return Err(Error::new_spanned(
                                attr_value,
                                "`repeats` must be positive",
                            ));
                        }
                    } else {
                        return Err(Error::new_spanned(name, "`repeats` defined twice"));
                    }
                }
                AttrInput::Int(IntAttrInput {
                    name,
                    value: Int::Skip(attr_value),
                    ..
                }) => {
                    if skip.is_none() {
                        let _value: u32 = attr_value.base10_parse()?;
                        skip = Some(attr_value);
                    } else {
                        return Err(Error::new_spanned(name, "`skip` defined twice"));
                    }
                }
                AttrInput::Type(TypeAttrInput {
                    name,
                    value: attr_value,
                    ..
                }) => {
                    if ty.is_none() {
                        ty = Some(attr_value)
                    } else {
                        return Err(Error::new_spanned(name, "`type` defined twice"));
                    }
                }
            };
        }
        if let Some(frequency) = frequency {
            if rate.is_none() {
                let value: NonZeroU32 = frequency.base10_parse().unwrap();
                if DEFAULT_RATE < value.get() {
                    return Err(Error::new_spanned(
                        frequency,
                        "`frequency` should be less than `rate`, which is 44100 Hz",
                    ));
                }
            }
            Ok(SineWaveAttrs {
                frequency,
                rate,
                len,
                repeats,
                skip,
                ty,
            })
        } else {
            Err(Error::new(input.span(), "`frequency` must be defined"))
        }
    }
}

struct Static {
    vis: Visibility,
    _static_token: Token![static],
    mutability: StaticMutability,
    ident: Ident,
    _eq_token: Token![=],
    name: Ident,
    _paren: Paren,
    attrs: SineWaveAttrs,
    _semi_token: Token![;],
}

impl Parse for Static {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Static {
            vis: input.parse()?,
            _static_token: input.parse()?,
            mutability: input.parse()?,
            ident: input.parse()?,
            _eq_token: input.parse()?,
            name: {
                let name: Ident = input.parse()?;
                if name != "sine_wave" {
                    return Err(Error::new(
                        name.span(),
                        "the identifier must be `sine_wave`",
                    ));
                }
                name
            },
            _paren: parenthesized!(content in input),
            attrs: content.parse()?,
            _semi_token: input.parse()?,
        })
    }
}

struct Const {
    vis: Visibility,
    _const_token: Token![const],
    ident: Ident,
    _eq_token: Token![=],
    name: Ident,
    _paren: Paren,
    attrs: SineWaveAttrs,
    _semi_token: Token![;],
}

impl Parse for Const {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Const {
            vis: input.parse()?,
            _const_token: input.parse()?,
            ident: input.parse()?,
            _eq_token: input.parse()?,
            name: {
                let name: Ident = input.parse()?;
                if name != "sine_wave" {
                    return Err(Error::new(
                        name.span(),
                        "the identifier must be `sine_wave`",
                    ));
                }
                name
            },
            _paren: parenthesized!(content in input),
            attrs: content.parse()?,
            _semi_token: input.parse()?,
        })
    }
}

enum SineWaveInput {
    Local(SineWaveAttrs),
    Static(Static),
    Const(Const),
}

impl Parse for SineWaveInput {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![pub]) && (input.peek2(Token![static]) || input.peek2(Token![const])) {
            if input.peek2(Token![static]) {
                input.parse().map(SineWaveInput::Static)
            } else {
                input.parse().map(SineWaveInput::Const)
            }
        } else if input.peek(Token![static]) {
            input.parse().map(SineWaveInput::Static)
        } else if input.peek(Token![const]) {
            input.parse().map(SineWaveInput::Const)
        } else {
            input.parse().map(SineWaveInput::Local)
        }
    }
}

fn get_number_of_samples(frequency: f64, rate: f64) -> usize {
    ((rate / frequency) as u64).try_into().unwrap()
}

impl SineWaveInput {
    fn get_attrs(&self) -> &SineWaveAttrs {
        match self {
            Self::Local(attrs) => attrs,
            Self::Static(Static { attrs, .. }) => attrs,
            Self::Const(Const { attrs, .. }) => attrs,
        }
    }
}

/// Generates an array of signed integers for a sine wave.
///
/// Sample rate and frequency of the wave can be controlled with `rate` and `frequency`
/// respectively. [Rounding][crate::sine_wave!#rounding] may apply which can affect the frequency
/// of the final wave slightly. The length of the array is calculated as `floor(rate / frequency)`.
///
/// The array is by default one period long so it can be repeated as many times as needed. If a
/// specific number of samples or number of repeated periods are required use `len` and `repeats`
/// respectively. Both cannot be used simultaneously. It is also possible to start the array on a
/// later point with `skip`. That effectively introduces a phase shift and defaults to zero skipped
/// samples.
///
/// # Arguments and examples
/// `frequency` selects the frequency of the sine wave, and it is the only required argument.
/// Negative or zero frequency is not accepted. It also must be sufficiently smaller than the
/// sampling rate used. See [Nyquist frequency][Nyquist_frequency] for more information. This macro
/// refuses to generate arrays with only zero values.
///
/// [Nyquist_frequency]: https://en.wikipedia.org/wiki/Nyquist_frequency
///
/// ```rust
/// # use sine_macro::sine_wave;
/// // Sine wave of 1,000 Hz with sampling rate of 44,100 Hz (the default).
/// let wave = sine_wave!(frequency: 1_000);
/// ```
///
/// `rate` specifies sampling rate of the array. If unspecified, 44,100 Hz is used instead.
/// Sampling rate must be sufficiently larger than the specified frequency of the wave. See the
/// information above about `frequency` for more information.
///
/// ```rust
/// # use sine_macro::sine_wave;
/// // Sine wave of 400 Hz with sampling rate of 48,000 Hz
/// let wave = sine_wave!(rate: 48_000, frequency: 400);
/// ```
///
/// `type` defines the data type of the array. It can be any of [`i8`], [`i16`] and [`i32`]. Defaults to
/// [`i16`] when unspecified. The values will always span the whole range of the type sans `MIN`.
///
/// ```rust
/// # use sine_macro::sine_wave;
/// // Sine wave of 100 Hz with i8 data type, so
/// let wave = sine_wave!(frequency: 100, type: i8);
/// ```
///
/// `len` specifies how many samples the array must contain. This may cut the wave short on any
/// period but it can be also used for generating waves of specific duration. E.g. one second long
/// wave can be generated by setting (sampling) `rate` and `len` to the same value. However the
/// same effect can be achieved with iterators (see [cycle][core::iter::Iterator::cycle] and
/// [take][core::iter::Iterator::take]) without storing longer arrays. `len` and `repeats` cannot
/// be used at the same time.
///
/// ```rust
/// # use sine_macro::sine_wave;
/// // Sine wave of 440 Hz with sampling rate of 16,000 Hz and one second length
/// let wave_of_a_second = sine_wave!(frequency: 440, rate: 16_000, len: 16_000);
/// // Or you can store only the first phase and use iterator tricks:
/// let wave = sine_wave!(frequency: 440, rate: 16_000);
/// let iter = wave.iter().cycle().take(16_000);
/// assert_eq!(wave_of_a_second.len(), iter.count());
/// ```
///
/// `repeats` specifies the number of periods to create. This can be useful if you know that you
/// need a number of repeats but cannot read the same array repeatedly. Otherwise this can be
/// achieved with iterators (see [cycle][core::iter::Iterator::cycle] and
/// [take][core::iter::Iterator::take]) without storing longer arrays. `len` and `repeats` cannot
/// be used at the same time.
///
/// ```rust
/// # use sine_macro::sine_wave;
/// // Sine wave of 360 Hz repeated 10 times
/// let wave_10_repeats = sine_wave!(frequency: 360, rate: 44_100, repeats: 10);
/// // Or you can store only the first phase and use iterator tricks:
/// let wave_no_repeats = sine_wave!(frequency: 360, rate: 44_100);
/// let iter = wave_no_repeats.iter().cycle().take(44_100 / 360 * 10);
/// assert_eq!(wave_10_repeats.len(), iter.count());
/// ```
///
/// `skip` specifies the number of samples to skip before starting to generate the array. This does
/// not affect the length of the array but it can be used for introducing a phase shift.
///
/// ```rust
/// # use sine_macro::sine_wave;
/// // Cosine wave of 400 Hz (cosine is sine with 90 degree phase shift, i.e. it starts 1/4 later)
/// let wave = sine_wave!(frequency: 400, skip: 100);
/// ```
///
/// # Use with static and const
/// Since `const` and `static` items must have their types defined and a macro cannot override
/// that, this provides a syntax similar to
/// [lazy_static!](https://docs.rs/lazy_static/latest/lazy_static/) for defining `const` or
/// `static` items. All the same arguments are supported with this syntax.
///
/// ```ignore
/// # use sine_macro::sine_wave;
/// // Syntax for const:
/// sine_wave! {
///     [pub] const NAME = sine_wave(frequency: ...);
/// }
/// // or static:
/// sine_wave! {
///     [pub] static [mut] NAME = sine_wave(frequency: ...);
/// }
/// ```
///
/// So, for example, to define public const item for a sine wave:
/// ```rust
/// # use sine_macro::sine_wave;
/// sine_wave! {
///     pub const BEEP = sine_wave(frequency: 440, rate: 48_000, type: i8);
/// }
/// ```
///
/// Or static mutable (for whatever reason):
/// ```rust
/// # use sine_macro::sine_wave;
/// sine_wave! {
///     static mut MUTABLE_BEEP = sine_wave(frequency: 440, rate: 48_000, type: i8);
/// }
/// ```
///
/// # Rounding
/// Rounding of the length of the array can affect the sine wave slightly. This always rounds
/// before generating the wave so the waves will always start from zero and end so that the next
/// value would be zero, unless `skip` or `len` is changing that.
///
/// ```rust
/// # use sine_macro::sine_wave;
/// // For example these waves are actually both 441 Hz
/// let wave_440 = sine_wave!(frequency: 440, rate: 44_100);
/// let wave_441 = sine_wave!(frequency: 441, rate: 44_100);
/// assert_eq!(wave_440, wave_441);
/// ```
#[proc_macro]
pub fn sine_wave(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as SineWaveInput);
    let attrs = input.get_attrs();
    let ty = attrs.ty.clone();
    let frequency: NonZeroU32 = attrs.frequency.clone().base10_parse().unwrap();
    let rate: NonZeroU32 = attrs
        .rate
        .clone()
        .map(|input| input.base10_parse().unwrap())
        .unwrap_or_else(|| NonZero::new(DEFAULT_RATE).unwrap());
    let values = get_number_of_samples(frequency.get() as f64, rate.get() as f64);
    let count;
    let sine_wave_tokens = {
        let multiplier = PI * 2_f64 / values as f64;
        let samples: Vec<_> = (0..values)
            .map(|i| (i as f64 * multiplier))
            .map(f64::sin)
            .map(|value| value * ty.max() as f64)
            .map(|value| value as i32)
            .collect();
        // Just a little sanity check
        if !samples.iter().any(|x| *x != 0) {
            return {
                Error::new_spanned(
                    &attrs.frequency,
                    format_args!(
                        "could not generate sine wave for `rate` of {} Hz and `frequency` of {} Hz",
                        rate, frequency
                    ),
                )
                .into_compile_error()
                .into()
            };
        }
        count = attrs
            .len
            .clone()
            .map(|input| input.base10_parse().unwrap())
            .unwrap_or_else(|| {
                samples.len()
                    * attrs
                        .repeats
                        .clone()
                        .map(|input| input.base10_parse().unwrap())
                        .unwrap_or(1)
            });
        let skip = attrs
            .skip
            .clone()
            .map(|input| input.base10_parse().unwrap())
            .unwrap_or(0);
        let tokens = TokenStream::from_iter(
            samples
                .iter()
                .cycle()
                .skip(skip)
                .take(count)
                .map(|value| TokenTree::Literal(ty.literal(*value)))
                .interleave(repeat_n(
                    TokenTree::from(Punct::new(',', Spacing::Alone)),
                    count - 1,
                )),
        );
        TokenStream::from(TokenTree::from(Group::new(Delimiter::Bracket, tokens)))
    };
    match input {
        SineWaveInput::Local(_) => sine_wave_tokens.into(),
        SineWaveInput::Static(item) => {
            assert_eq!(item.name, "sine_wave");
            let vis = item.vis;
            let mutability = item.mutability;
            let ident = item.ident;
            let ty = ty.ident();
            quote! {
                #vis static #mutability #ident: [#ty; #count] = #sine_wave_tokens;
            }
            .into()
        }
        SineWaveInput::Const(item) => {
            assert_eq!(item.name, "sine_wave");
            let vis = item.vis;
            let ident = item.ident;
            let ty = ty.ident();
            quote! {
                #vis const #ident: [#ty; #count] = #sine_wave_tokens;
            }
            .into()
        }
    }
}
