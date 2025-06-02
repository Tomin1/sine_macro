/*
 * Copyright (c) 2025 Tomi Leppänen
 * SPDX-License-Identifier: MIT
 */

//! A procedural macro for generating sine waves.
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
use proc_macro2::{Delimiter, Group, Literal, Punct, Spacing, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use std::f64::consts::PI;
use std::iter::repeat_n;
use syn::parse::{Error, Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{Ident, LitInt, Result, StaticMutability, Visibility, parse_macro_input};
use syn::{Token, parenthesized};

struct AttrInput {
    name: Ident,
    _sep: Token![:],
    value: LitInt,
}

impl Parse for AttrInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(AttrInput {
            name: input.parse()?,
            _sep: input.parse()?,
            value: input.parse()?,
        })
    }
}

#[derive(Clone)]
struct SineWaveAttrs {
    frequency: Option<LitInt>,
    rate: Option<LitInt>,
    len: Option<LitInt>,
    repeats: Option<LitInt>,
}

impl Parse for SineWaveAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = Punctuated::<AttrInput, Token![,]>::parse_terminated(input)?;
        let mut frequency = None;
        let mut rate = None;
        let mut len = None;
        let mut repeats = None;
        for attr in attrs {
            if attr.name == "frequency" {
                if frequency.is_none() {
                    let value: u32 = attr.value.base10_parse()?;
                    if value > 0 {
                        frequency = Some(attr.value)
                    } else {
                        return Err(Error::new(
                            attr.value.span(),
                            "`frequency` must be positive",
                        ));
                    }
                } else {
                    return Err(Error::new(attr.name.span(), "`frequency` defined twice"));
                }
            } else if attr.name == "rate" {
                if rate.is_none() {
                    let value: u32 = attr.value.base10_parse()?;
                    if value > 0 {
                        rate = Some(attr.value)
                    } else {
                        return Err(Error::new(attr.value.span(), "`rate` must be positive"));
                    }
                } else {
                    return Err(Error::new(attr.name.span(), "`rate` defined twice"));
                }
            } else if attr.name == "len" {
                if repeats.is_some() {
                    return Err(Error::new(
                        attr.name.span(),
                        "cannot define both `len` and `repeats`",
                    ));
                } else if len.is_none() {
                    let value: usize = attr.value.base10_parse()?;
                    if value > 0 {
                        len = Some(attr.value)
                    } else {
                        return Err(Error::new(attr.value.span(), "`len` must be positive"));
                    }
                } else {
                    return Err(Error::new(attr.name.span(), "`len` defined twice"));
                }
            } else if attr.name == "repeats" {
                if len.is_some() {
                    return Err(Error::new(
                        attr.name.span(),
                        "cannot define both `len` and `repeats`",
                    ));
                } else if repeats.is_none() {
                    let value: usize = attr.value.base10_parse()?;
                    if value > 0 {
                        repeats = Some(attr.value)
                    } else {
                        return Err(Error::new(attr.value.span(), "`repeats` must be positive"));
                    }
                } else {
                    return Err(Error::new(attr.name.span(), "`repeats` defined twice"));
                }
            } else {
                let idents = {
                    let mut idents = Vec::new();
                    if frequency.is_none() {
                        idents.push("`frequency`");
                    }
                    if rate.is_none() {
                        idents.push("`rate`");
                    }
                    if len.is_none() && repeats.is_none() {
                        idents.push("`len`");
                        idents.push("`repeats`");
                    }
                    idents
                };
                let text = if idents.is_empty() {
                    "unexpected identifier".to_string()
                } else {
                    format_args!("expected any of {}", idents.join(", ")).to_string()
                };
                return Err(Error::new(attr.name.span(), text));
            }
        }
        Ok(SineWaveAttrs {
            frequency,
            rate,
            len,
            repeats,
        })
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

fn get_number_of_samples(frequency: f64, rate: f64) -> std::result::Result<usize, (u64, u64)> {
    if frequency > rate {
        Err((frequency as u64, rate as u64))
    } else {
        Ok(((rate / frequency) as u64).try_into().unwrap())
    }
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

/// Generates an array of [`i16`] for a sine wave.
///
/// Sample rate and frequency of the wave can be controlled with `rate` and `frequency`
/// respectively. Rounding may apply which can affect the frequency of the final wave slightly.
///
/// The array is by default one period long so it can be repeated as many times as needed.
/// If a specific number of samples or number of repeated periods are required use `len` and
/// `repeats` respectively. Both cannot be used simultaneously.
///
/// If `rate` is not selected, it defaults to 44,100 Hz, and `frequency` defaults to 440 Hz.
///
/// # Examples
/// This defines custom sine wave with sampling rate of 48,000 Hz at frequency of 1000 Hz with only
/// one repeat (the default).
///
/// ```rust
/// # use sine_macro::sine_wave;
/// let wave = sine_wave!(rate: 48_000, frequency: 1000, repeats: 1);
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
    let frequency: u32 = attrs
        .frequency
        .clone()
        .map(|input| input.base10_parse().unwrap())
        .unwrap_or(440);
    let rate: u32 = attrs
        .rate
        .clone()
        .map(|input| input.base10_parse().unwrap())
        .unwrap_or(44_100);
    let values = get_number_of_samples(frequency as f64, rate as f64);
    let count;
    let sine_wave_tokens = match values {
        Err((ref frequency, ref rate)) => match (attrs.frequency.clone(), attrs.rate.clone()) {
            (Some(freq), _) => {
                let error =
                    format_args!("`frequency` should be less than rate, which is {} Hz", rate)
                        .to_string();
                return quote_spanned!(freq.span() => compile_error!(#error)).into();
            }
            (None, Some(rate)) => {
                let error = format_args!(
                    "`rate` should be more than frequency, which is {} Hz",
                    frequency
                )
                .to_string();
                return quote_spanned!(rate.span() => compile_error!(#error)).into();
            }
            _ => unreachable!(),
        },
        Ok(values) => {
            let multiplier = PI * 2_f64 / values as f64;
            let samples: Vec<_> = (0..values)
                .map(|i| (i as f64 * multiplier))
                .map(f64::sin)
                .map(|value| value * i16::MAX as f64)
                .map(|value| value as i16)
                .collect();
            if !samples.iter().any(|x| *x != 0) {
                return match (attrs.frequency.clone(), attrs.rate.clone()) {
                    (Some(freq), _) => {
                        let error = format_args!(
                            "could not generate sine wave for `rate` of {} Hz and frequency of {} Hz",
                            rate, frequency
                        )
                        .to_string();
                        quote_spanned!(freq.span() => compile_error!(#error)).into()
                    }
                    (None, Some(rate)) => {
                        let error = format_args!(
                            "could not generate sine wave for `rate` of {} Hz and frequency of {} Hz",
                            rate, frequency
                        )
                        .to_string();
                        quote_spanned!(rate.span() => compile_error!(#error)).into()
                    }
                    _ => unreachable!(),
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
            let tokens = TokenStream::from_iter(
                samples
                    .iter()
                    .cycle()
                    .take(count)
                    .map(|value| TokenTree::Literal(Literal::i16_suffixed(*value)))
                    .interleave(repeat_n(
                        TokenTree::from(Punct::new(',', Spacing::Alone)),
                        count - 1,
                    )),
            );
            TokenStream::from(TokenTree::from(Group::new(Delimiter::Bracket, tokens)))
        }
    };
    match input {
        SineWaveInput::Local(_) => sine_wave_tokens.into(),
        SineWaveInput::Static(item) => {
            assert_eq!(item.name, "sine_wave");
            let vis = item.vis;
            let mutability = item.mutability;
            let ident = item.ident;
            quote! {
                #vis static #mutability #ident: [i16; #count] = #sine_wave_tokens;
            }
            .into()
        }
        SineWaveInput::Const(item) => {
            assert_eq!(item.name, "sine_wave");
            let vis = item.vis;
            let ident = item.ident;
            quote! {
                #vis const #ident: [i16; #count] = #sine_wave_tokens;
            }
            .into()
        }
    }
}
