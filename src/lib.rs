/*
 * Copyright (c) 2025 Tomi Leppänen
 * SPDX-License-Identifier: MIT
 */

use itertools::Itertools;
use proc_macro::{Delimiter, Group, Literal, Punct, Spacing, TokenStream, TokenTree};
use quote::quote_spanned;
use std::f64::consts::PI;
use std::iter::repeat_n;
use std::ops::Neg;
use syn::parse::{Error, Parse, ParseStream};
use syn::{Ident, LitInt, Token, punctuated::Punctuated};
use syn::{Result, parse_macro_input};

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

struct SineWaveInput {
    frequency: Option<LitInt>,
    rate: Option<LitInt>,
}

impl Parse for SineWaveInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = Punctuated::<AttrInput, Token![,]>::parse_separated_nonempty(input)?;
        let mut frequency = None;
        let mut rate = None;
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
            } else {
                let text = match (frequency, rate) {
                    (None, None) => "expected `frequency` or `rate`",
                    (Some(_), None) => "expected `rate`",
                    (None, Some(_)) => "expected `frequency`",
                    (Some(_), Some(_)) => "unexpected token",
                };
                return Err(Error::new(attr.name.span(), text));
            }
        }
        Ok(SineWaveInput { frequency, rate })
    }
}

fn get_number_of_unique_samples(
    frequency: f64,
    rate: f64,
) -> std::result::Result<usize, (u64, u64)> {
    if frequency > rate / 4.0 {
        Err((frequency as u64, rate as u64))
    } else {
        Ok(((rate / frequency / 4.0 + 1.0) as u64).try_into().unwrap())
    }
}

// TODO: Alternate parsing "mode" for const and static variables
// TODO: Add [symmetric: true|false] to select between this and true sine wave
// TODO: Add length to select how many samples to generate
// TODO: Document rounding
#[proc_macro]
pub fn sine_wave(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as SineWaveInput);
    let values = get_number_of_unique_samples(
        input
            .frequency
            .clone()
            .map(|input| input.base10_parse().unwrap())
            .unwrap_or(440)
            .into(),
        input
            .rate
            .clone()
            .map(|input| input.base10_parse().unwrap())
            .unwrap_or(44_100)
            .into(),
    );
    if let Err((ref frequency, ref rate)) = values {
        if let Some(freq) = input.frequency {
            let quarter = rate / 4;
            let error = format_args!(
                "`frequency` should be less than {} for rate of {} Hz",
                quarter, rate
            )
            .to_string();
            quote_spanned!(freq.span() => compile_error!(#error)).into()
        } else if let Some(rate) = input.rate {
            let quadruple = frequency * 4;
            let error = format_args!(
                "`rate` should be more than {} for frequency of {} Hz",
                quadruple, frequency
            )
            .to_string();
            quote_spanned!(rate.span() => compile_error!(#error)).into()
        } else {
            unreachable!()
        }
    } else if let Ok(values) = values {
        let multiplier = PI * 2_f64 / ((values - 1) * 4) as f64;
        let samples: Vec<_> = (0..values)
            .chain((1..values - 1).rev())
            .map(|i| (i as f64 * multiplier))
            .map(f64::sin)
            .map(|value| value * i16::MAX as f64)
            .map(|value| value as i16)
            .collect();
        let samples: Vec<_> = samples
            .iter()
            .map(Clone::clone)
            .chain(samples.iter().map(Neg::neg))
            .collect();
        let tokens = TokenStream::from_iter(
            samples
                .iter()
                .map(|value| TokenTree::Literal(Literal::i16_suffixed(*value)))
                .interleave(repeat_n(
                    TokenTree::from(Punct::new(',', Spacing::Alone)),
                    samples.len() - 1,
                )),
        );
        TokenStream::from(TokenTree::from(Group::new(Delimiter::Bracket, tokens)))
    } else {
        unreachable!()
    }
}
