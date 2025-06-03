/*
 * Copyright (c) 2025 Tomi Leppänen
 * SPDX-License-Identifier: MIT
 */

//! A procedural macro for generating signed integer sine waves.
//!
//! # Example
//! ```rust
//! use sine_macro::sine_wave;
//!
//! // Sine wave defined as const:
//! sine_wave! {
//!     const MY_CONST_SINE_WAVE = sine_wave(frequency: 400, rate: 48_000);
//! }
//!
//! // Or as static variable:
//! sine_wave! {
//!     static MY_STATIC_SINE_WAVE = sine_wave(frequency: 1000, rate: 48_000, len: 48_000);
//! }
//!
//! // Sine wave defined as local variable with default rate of 44,100 Hz:
//! let wave = sine_wave!(frequency: 800, repeats: 10);
//! ```

#![deny(missing_docs)]

use itertools::Itertools;
use proc_macro2::{Delimiter, Group, Punct, Spacing, TokenStream, TokenTree};
use quote::quote;
use std::f64::consts::PI;
use std::iter::repeat_n;
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
                        let value: u32 = attr_value.base10_parse()?;
                        if value > 0 {
                            if let Some(rate) = &rate {
                                let rate: u32 = rate.base10_parse().unwrap();
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
                            return Err(Error::new_spanned(
                                attr_value,
                                "`frequency` must be positive",
                            ));
                        }
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
                        let value: u32 = attr_value.base10_parse()?;
                        if value > 0 {
                            if let Some(frequency) = &frequency {
                                let frequency: u32 = frequency.base10_parse().unwrap();
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
                            return Err(Error::new_spanned(attr_value, "`rate` must be positive"));
                        }
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
                        let value: usize = attr_value.base10_parse()?;
                        if value > 0 {
                            len = Some(attr_value)
                        } else {
                            return Err(Error::new_spanned(attr_value, "`len` must be positive"));
                        }
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
                let value: u32 = frequency.base10_parse().unwrap();
                if DEFAULT_RATE < value {
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
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![static]) {
            input.parse().map(SineWaveInput::Static)
        } else if lookahead.peek(Token![const]) {
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
/// respectively. Rounding may apply which can affect the frequency of the final wave slightly.
///
/// The array is by default one period long so it can be repeated as many times as needed.
/// If a specific number of samples or number of repeated periods are required use `len` and
/// `repeats` respectively. Both cannot be used simultaneously. It is also possible to start the
/// array on a later point with `skip`. That effectively introduces a phase shift and defaults to
/// zero skipped samples.
///
/// If `rate` is not selected, it defaults to 44,100 Hz. `frequency` must be defined always.
///
/// Type can be chosen with `type`. It can be any of [`i8`], [`i16`] and [`i32`]. Defaults to [`i16`].
///
/// # Examples
/// This defines custom sine wave with sampling rate of 48,000 Hz at frequency of 1000 Hz with i32
/// type.
///
/// ```rust
/// # use sine_macro::sine_wave;
/// let wave = sine_wave!(rate: 48_000, frequency: 1_000, type: i32);
/// ```
///
/// You can also use this to define `static` or `const` variables. This defines one second long
/// sine wave with sampling rate of 8,000 Hz and frequency of 400 Hz.
///
/// ```rust
/// # use sine_macro::sine_wave;
/// sine_wave! {
///     static ONE_SECOND = sine_wave(frequency: 400, rate: 8_000, len: 8_000);
/// }
/// ```
#[proc_macro]
pub fn sine_wave(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as SineWaveInput);
    let attrs = input.get_attrs();
    let ty = attrs.ty.clone();
    let frequency: u32 = attrs.frequency.clone().base10_parse().unwrap();
    let rate: u32 = attrs
        .rate
        .clone()
        .map(|input| input.base10_parse().unwrap())
        .unwrap_or(DEFAULT_RATE);
    let values = get_number_of_samples(frequency as f64, rate as f64);
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
