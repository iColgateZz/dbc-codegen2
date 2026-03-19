use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::File;

use crate::DbcFile;
use crate::ir::message::{Message, MessageId};
use crate::ir::signal::Signal;
use crate::ir::signal_layout::SignalLayout;
use crate::ir::signal_value_type::{RustIntegerLiteral, RustType, RustFloatLiteral, IntReprType};

pub struct RustGen;

impl RustGen {
    pub fn generate(file: &DbcFile) -> String {
        let imports = quote! {
            use embedded_can::{Frame, Id, StandardId, ExtendedId};
            use bitvec::prelude::*;
        };

        let messages = &file.messages;
        let error_enum = ErrorEnum;
        let msg_trait = MsgTrait;
        let msg_enum = MsgEnum { messages };
        let message_defs: Vec<_> = messages.iter().map(|m| MessageDef { msg: m, file }).collect();

        let tokens = quote! {
            #imports

            #error_enum

            #msg_trait

            #msg_enum

            #( #message_defs )*
        };

        let file: File = syn::parse2(tokens).unwrap();
        prettyplease::unparse(&file)
    }
}

struct ErrorEnum;

impl ToTokens for ErrorEnum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote! {
            #[derive(Debug, Clone)]
            pub enum CanError {
                Err1,
                Err2,
                InvalidPayloadSize,
                ValueOutOfRange,
            }
        }
        .to_tokens(tokens);
    }
}

struct MsgTrait;

impl ToTokens for MsgTrait {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote! {
            pub trait CanMessage<const LEN: usize>: Sized {
                fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError>;
                fn encode(&self) -> [u8; LEN];
            }
        }
        .to_tokens(tokens);
    }
}

struct MsgEnum<'a> {
    messages: &'a [Message],
}

impl ToTokens for MsgEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variants = self.messages.iter().map(|msg| {
            let name = format_ident!("{}", msg.name.upper_camel());
            quote! { #name(#name) }
        });

        quote! {
            #[derive(Debug, Clone)]
            pub enum Msg {
                #( #variants, )*
            }
        }
        .to_tokens(tokens);

        let arms = self.messages.iter().map(|msg| {
            let name = format_ident!("{}", msg.name.upper_camel());
            quote! { #name::ID => Msg::#name(#name::try_from_frame(frame)?) }
        });

        quote! {
            impl Msg {
                fn try_from(frame: &impl Frame) -> Result<Self, CanError> {
                    let result = match frame.id() {
                        #( #arms, )*
                        _ => return Err(CanError::Err1),
                    };

                    Ok(result)
                }
            }
        }
        .to_tokens(tokens);
    }
}

struct MessageDef<'a> {
    msg: &'a Message,
    file: &'a DbcFile,
}

impl ToTokens for MessageDef<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let msg = self.msg;
        let name = format_ident!("{}", msg.name.upper_camel());

        let signals: Vec<SignalCtx> = msg
            .signal_idxs
            .iter()
            .map(|idx| SignalCtx::new(&self.file.signals[idx.0], self.file))
            .collect();

        let value_enums = signals
            .iter()
            .map(|s| SignalValueEnum { signal: s.signal });

        let fields = signals.iter().map(|s| {
            let field = s.field_ident();
            let ty = s.rust_type();

            quote! { pub #field: #ty }
        });

        let id_expr = match msg.id {
            MessageId::Standard(id) => {
                quote! {
                    Id::Standard(unsafe { StandardId::new_unchecked(#id) })
                }
            }
            MessageId::Extended(id) => {
                quote! {
                    Id::Extended(unsafe { ExtendedId::new_unchecked(#id) })
                }
            }
        };

        let len = msg.size as usize;

        let constructor_params = signals.iter().map(|s| {
            let field = s.field_ident();
            let ty = s.rust_type();

            quote! { #field: #ty }
        });

        let constructor_fields = signals.iter().map(|s| s.field_ident());

        let constructor_validations = signals.iter().map(|s| {
            let field = s.field_ident();
            let setter = s.setter_ident();

            quote! {
                msg.#setter(msg.#field)?;
            }
        });

        let getters = signals.iter().map(|s| {
            let field = s.field_ident();
            let ty = s.rust_type();

            quote! {
                pub fn #field(&self) -> #ty {
                    self.#field
                }
            }
        });

        let setters = signals.iter().map(|s| {
            let field = s.field_ident();
            let setter = s.setter_ident();
            let ty = s.rust_type();
            let check = s.range_check();

            quote! {
                pub fn #setter(&mut self, value: #ty) -> Result<(), CanError> {
                    #check
                    self.#field = value;
                    Ok(())
                }
            }
        });

        let impl_block = quote! {
            impl #name {
                pub const ID: Id = #id_expr;
                pub const LEN: usize = #len;

                pub fn new(
                    #( #constructor_params ),*
                ) -> Result<Self, CanError> {
                    let mut msg = Self {
                        #( #constructor_fields ),*
                    };

                    #( #constructor_validations )*

                    Ok(msg)
                }

                #( #getters )*

                #( #setters )*
            }
        };

        let try_from = {
            let reads = signals.iter().map(|s| s.decode_read());
            let fields = signals.iter().map(|s| s.decode_field());

            quote! {
                fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
                    let data = frame.data();
                    if data.len() < Self::LEN {
                        return Err(CanError::InvalidPayloadSize);
                    }

                    #( #reads )*

                    Ok(Self {
                        #( #fields, )*
                    })
                }
            }
        };

        let encode = {
            let writes = signals.iter().map(|s| s.encode_write());

            quote! {
                fn encode(&self) -> [u8; #name::LEN] {
                    let mut data = [0u8; #name::LEN];

                    #( #writes )*

                    data
                }
            }
        };

        quote! {
            #( #value_enums )*

            #[derive(Debug, Clone)]
            pub struct #name {
                #( #fields, )*
            }

            #impl_block

            impl CanMessage<{ #name::LEN }> for #name {
                #try_from
                #encode
            }
        }
        .to_tokens(tokens);
    }
}
struct SignalValueEnum<'a> {
    signal: &'a Signal,
}

impl ToTokens for SignalValueEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let signal = self.signal;

        let Some(enum_def) = &signal.signal_value_enum else {
            return;
        };

        let enum_name = format_ident!("{}", signal.name.upper_camel());
        let repr_type = &signal.physical_type;
        let rust_type = format_ident!("{}", repr_type.as_rust_type());

        let variants = enum_def.variants.iter().map(|vd| {
            let name = format_ident!("{}", vd.description);
            quote! { #name }
        });

        let from_arms = enum_def.variants.iter().map(|vd| {
            let name = format_ident!("{}", vd.description);
            let value = repr_type.literal(vd.value);
            quote! { #value => Self::#name }
        });

        let into_arms = enum_def.variants.iter().map(|vd| {
            let name = format_ident!("{}", vd.description);
            let value = repr_type.literal(vd.value);
            quote! { #enum_name::#name => #value }
        });

        quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum #enum_name {
                #( #variants, )*
                _Other(#rust_type),
            }

            impl From<#rust_type> for #enum_name {
                fn from(val: #rust_type) -> Self {
                    match val {
                        #( #from_arms, )*
                        _ => Self::_Other(val),
                    }
                }
            }

            impl From<#enum_name> for #rust_type {
                fn from(val: #enum_name) -> Self {
                    match val {
                        #( #into_arms, )*
                        #enum_name::_Other(v) => v,
                    }
                }
            }
        }
        .to_tokens(tokens);
    }
}

struct SignalCtx<'a> {
    signal: &'a Signal,
    layout: &'a SignalLayout,
}

impl<'a> SignalCtx<'a> {
    fn new(signal: &'a Signal, file: &'a DbcFile) -> Self {
        Self {
            signal,
            layout: &file.signal_layouts[signal.layout.0],
        }
    }

    fn field_ident(&self) -> syn::Ident {
        format_ident!("{}", self.signal.name.lower())
    }

    fn setter_ident(&self) -> syn::Ident {
        format_ident!("set_{}", self.signal.name.lower())
    }

    fn raw_ident(&self) -> syn::Ident {
        format_ident!("raw_{}", self.signal.name.lower())
    }

    fn enum_ident(&self) -> syn::Ident {
        format_ident!("{}", self.signal.name.upper_camel())
    }

    fn rust_type(&self) -> syn::Ident {
        if self.signal.signal_value_enum.is_some() {
            self.enum_ident()
        } else {
            format_ident!("{}", self.signal.physical_type.as_rust_type())
        }
    }

    fn raw_rust_type(&self) -> syn::Ident {
        format_ident!("{}", self.signal.physical_type.as_rust_type())
    }

    fn is_enum(&self) -> bool {
        self.signal.signal_value_enum.is_some()
    }

    fn range_check(&self) -> TokenStream {
        if self.is_enum() {
            return quote! {};
        }

        let phys = &self.signal.physical_type;
        let min = self.layout.min;
        let max = self.layout.max;

        let min_literal: TokenStream;
        let max_literal: TokenStream;

        if phys.is_float() {
            min_literal = phys.fliteral(min).to_token_stream();
            max_literal = phys.fliteral(max).to_token_stream();
        } else {
            min_literal = phys.literal(min as i64).to_token_stream();
            max_literal = phys.literal(max as i64).to_token_stream();
        }

        quote! {
            if value < #min_literal || value > #max_literal {
                return Err(CanError::ValueOutOfRange);
            }
        }
    }

    fn start_bit(&self) -> usize {
        self.layout.start_bit as usize
    }

    fn end_bit(&self) -> usize {
        self.start_bit() + self.layout.size as usize
    }

    fn decode_read(&self) -> TokenStream {
        let raw = self.raw_ident();
        let start = self.start_bit();
        let end = self.end_bit();

        if self.is_enum() || !self.signal.physical_type.is_float() {
            let raw_ty = self.raw_rust_type();
            quote! {
                let #raw = data.view_bits::<Lsb0>()[#start..#end].load_le::<#raw_ty>();
            }
        } else {
            let int_ty = IntReprType::from_size_sign(self.layout.size, false);
            let int_ty_ident = format_ident!("{}", int_ty.as_rust_type());

            quote! {
                let #raw = data.view_bits::<Lsb0>()[#start..#end].load_le::<#int_ty_ident>();
            }
        }
    }

    //TODO: do not perform multiplication when factor is 1
    //      do not perform addition when offset is 0
    fn decode_field(&self) -> TokenStream {
        let field = self.field_ident();
        let raw = self.raw_ident();

        if self.is_enum() {
            let enum_name = self.enum_ident();
            let raw_ty = self.raw_rust_type();
            quote! { #field: #enum_name::from(#raw as #raw_ty) }
        } else if self.signal.physical_type.is_float() {
            let factor = self.layout.factor;
            let offset = self.layout.offset;
            let phys = &self.signal.physical_type;

            let factor_literal = phys.fliteral(factor).to_token_stream();
            let offset_literal = phys.fliteral(offset).to_token_stream();

            let ty = format_ident!("{}", phys.as_rust_type());

            quote! {
                #field: (#raw as #ty) * (#factor_literal) + (#offset_literal)
            }
        } else {
            let factor = self.layout.factor;
            let offset = self.layout.offset;
            let phys = &self.signal.physical_type;

            let factor_literal = phys.literal(factor as i64).to_token_stream();
            let offset_literal = phys.literal(offset as i64).to_token_stream();

            quote! {
                #field: (#raw) * (#factor_literal) + (#offset_literal)
            }
        }
    }

    //TODO: do not perform division when factor is 1
    //      do not perform subtraction when offset is 0
    fn encode_write(&self) -> TokenStream {
        let field = self.field_ident();
        let start = self.start_bit();
        let end = self.end_bit();

        if self.is_enum() {
            let ty = self.raw_rust_type();
            quote! {
                data.view_bits_mut::<Lsb0>()[#start..#end].store_le(#ty::from(self.#field));
            }
        } else if self.signal.physical_type.is_float() {
            let phys = &self.signal.physical_type;
            let factor = self.layout.factor;
            let offset = self.layout.offset;

            let factor_literal = phys.fliteral(factor).to_token_stream();
            let offset_literal = phys.fliteral(offset).to_token_stream();

            let int_ty = IntReprType::from_size_sign(self.layout.size, false);
            let int_ty_ident = format_ident!("{}", int_ty.as_rust_type());

            quote! {
                data.view_bits_mut::<Lsb0>()[#start..#end].store_le(((self.#field - (#offset_literal)) / (#factor_literal)) as #int_ty_ident);
            }
        } else {
            let factor = self.layout.factor;
            let offset = self.layout.offset;
            let phys = &self.signal.physical_type;
            
            let factor_literal = phys.literal(factor as i64).to_token_stream();
            let offset_literal = phys.literal(offset as i64).to_token_stream();
            
            quote! {
                data.view_bits_mut::<Lsb0>()[#start..#end].store_le((self.#field - (#offset_literal)) / (#factor_literal));
            }
        }
    }

}
