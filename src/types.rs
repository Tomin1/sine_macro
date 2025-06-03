/*
 * Copyright (c) 2025 Tomi Leppänen
 * SPDX-License-Identifier: MIT
 */

use syn::parse::{Error, Parse, ParseStream};
use syn::{Ident, LitInt, Result, Token};

pub(crate) enum Int {
    Frequency(LitInt),
    Rate(LitInt),
    Len(LitInt),
    Repeats(LitInt),
    Skip(LitInt),
}

#[derive(Clone)]
pub(crate) enum Type {
    I8(Ident),
    I16(Ident),
    I32(Ident),
}

pub(crate) struct IntAttrInput {
    pub name: Ident,
    _sep: Token![:],
    pub value: Int,
}

pub(crate) struct TypeAttrInput {
    pub name: Token![type],
    _sep: Token![:],
    pub value: Type,
}

pub(crate) enum AttrInput {
    Int(IntAttrInput),
    Type(TypeAttrInput),
}

impl Parse for Type {
    fn parse(input: ParseStream) -> Result<Self> {
        let value: Ident = input.parse()?;
        match value.to_string().as_ref() {
            "i8" => Ok(Type::I8(value)),
            "i16" => Ok(Type::I16(value)),
            "i32" => Ok(Type::I32(value)),
            _ => Err(Error::new_spanned(
                value,
                "invalid value for `type`, must be one of `i8`, `i16` and `i32`",
            )),
        }
    }
}

impl Parse for AttrInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![type]) {
            Ok(AttrInput::Type(TypeAttrInput {
                name: input.parse()?,
                _sep: input.parse()?,
                value: input.parse()?,
            }))
        } else {
            let name: Ident = input.parse()?;
            Ok(AttrInput::Int(IntAttrInput {
                name: name.clone(),
                _sep: input.parse()?,
                value: match name.to_string().as_ref() {
                    "frequency" => input.parse().map(Int::Frequency)?,
                    "rate" => input.parse().map(Int::Rate)?,
                    "len" => input.parse().map(Int::Len)?,
                    "repeats" => input.parse().map(Int::Repeats)?,
                    "skip" => input.parse().map(Int::Skip)?,
                    _ => {
                        return Err(Error::new(
                            name.span(),
                            "invalid identifier, must be one of `frequency`, `rate`, `len`, `repeats`, `skip` and `type`",
                        ));
                    }
                },
            }))
        }
    }
}

pub(crate) mod helpers {
    use crate::types::Type;
    use proc_macro2::Span;

    pub(crate) trait Ident {
        fn ident(&self) -> syn::Ident;
    }

    impl Ident for Type {
        fn ident(&self) -> syn::Ident {
            match self {
                Self::I8(ident) => ident.clone(),
                Self::I16(ident) => ident.clone(),
                Self::I32(ident) => ident.clone(),
            }
        }
    }

    impl<T: Ident> Ident for Option<T> {
        fn ident(&self) -> syn::Ident {
            match self {
                Some(item) => item.ident(),
                None => syn::Ident::new(crate::DEFAULT_TYPE, Span::call_site()),
            }
        }
    }

    pub(crate) trait Literal {
        fn literal(&self, value: i32) -> proc_macro2::Literal;
    }

    impl Literal for Type {
        fn literal(&self, value: i32) -> proc_macro2::Literal {
            match self {
                Type::I8(_) => proc_macro2::Literal::i8_suffixed(value as i8),
                Type::I16(_) => proc_macro2::Literal::i16_suffixed(value as i16),
                Type::I32(_) => proc_macro2::Literal::i32_suffixed(value),
            }
        }
    }

    impl<T: Literal> Literal for Option<T> {
        fn literal(&self, value: i32) -> proc_macro2::Literal {
            match self {
                Some(item) => item.literal(value),
                None => proc_macro2::Literal::i16_suffixed(value as i16),
            }
        }
    }

    pub(crate) trait Max {
        fn max(&self) -> i32;
    }

    impl Max for Type {
        fn max(&self) -> i32 {
            match self {
                Self::I8(_) => i8::MAX as i32,
                Self::I16(_) => i16::MAX as i32,
                Self::I32(_) => i32::MAX,
            }
        }
    }

    impl<T: Max> Max for Option<T> {
        fn max(&self) -> i32 {
            match self {
                Some(item) => item.max(),
                None => i16::MAX as i32,
            }
        }
    }
}
