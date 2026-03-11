use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::File;

use crate::ir::message::{Message, MessageId};
use crate::ir::signal::{Signal};
use crate::ir::helpers::ToUpperCamelCase;

pub struct RustGen;

impl RustGen {
    pub fn generate(messages: &[Message]) -> String {
        let imports = quote! {
            use embedded_can::{Frame, Id, StandardId, ExtendedId};
        };

        let error_enum = ErrorEnum;
        let msg_trait = MsgTrait;
        let msg_enum = MsgEnum { messages };
        let message_defs: Vec<_> = messages.iter().map(|m| MessageDef { msg: m }).collect();

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
        tokens.extend(quote! {
            #[derive(Debug, Clone)]
            pub enum CanError {
                Err1,
                Err2,
            }
        });
    }
}

struct MsgTrait;

impl ToTokens for MsgTrait {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! {
            pub trait CanMessage<const LEN: usize>: Sized {
                fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError>;
                fn encode(&self) -> (Id, [u8; LEN]);
            }
        });
    }
}

struct MsgEnum<'a> {
    messages: &'a [Message],
}

impl ToTokens for MsgEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variants = self.messages.iter().map(|msg| {
            let name = format_ident!("{}", msg.name.0);
            quote! { #name(#name) }
        });

        tokens.extend(quote! {
            #[derive(Debug, Clone)]
            pub enum Msg {
                #( #variants, )*
            }
        });

        let arms = self.messages.iter().map(|msg| {
            let name = format_ident!("{}", msg.name.0);
            quote! { #name::ID => Msg::#name(#name::try_from_frame(frame)?) }
        });

        tokens.extend(quote! {
            impl Msg {
                fn try_from(frame: &impl Frame) -> Result<Self, CanError> {
                    let id = match frame.id() {
                        Id::Standard(sid) => sid.as_raw() as u32,
                        Id::Extended(eid) => eid.as_raw(),
                    };

                    let result = match id {
                        #( #arms, )*
                        _ => return Err(CanError::Err1),
                    };

                    Ok(result)
                }
            }
        });
    }
}

struct MessageDef<'a> {
    msg: &'a Message,
}

impl ToTokens for MessageDef<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let msg = self.msg;
        let name = format_ident!("{}", msg.name.0);

        let value_enums = msg.signals.iter().map(|s| SignalValueEnum { signal: s });

        let fields = msg.signals.iter().map(|sig| {
            let field = format_ident!("{}", sig.name.0.0);
            quote! { pub #field: f64 }
        });

        let id = match msg.id {
            MessageId::Standard(id) => id as u32,
            MessageId::Extended(id) => id,
        };
        let len = msg.size as usize;

        let impl_block = quote! {
            impl #name {
                pub const ID: u32 = #id;
                pub const LEN: usize = #len;
            }
        };

        let try_from = {
            let mut byte = 0usize;
            let reads = msg.signals.iter().map(|sig| {
                let raw = format_ident!("raw_{}", sig.name.0.0);
                let b0 = byte;
                let b1 = byte + 1;
                byte += 2;
                quote! { let #raw = u16::from_le_bytes([data[#b0], data[#b1]]); }
            });

            let fields = msg.signals.iter().map(|sig| {
                let field = format_ident!("{}", sig.name.0.0);
                let raw = format_ident!("raw_{}", sig.name.0.0);
                let factor = sig.factor;
                quote! { #field: #raw as f64 * #factor }
            });

            quote! {
                fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
                    let data = frame.data();

                    #( #reads )*

                    Ok(Self {
                        #( #fields, )*
                    })
                }
            }
        };

        let encode = {
            let id_expr = match msg.id {
                MessageId::Standard(_) => {
                    quote! { Id::Standard(StandardId::new(Self::ID as u16).unwrap()) }
                }
                MessageId::Extended(_) => {
                    quote! { Id::Extended(ExtendedId::new(Self::ID).unwrap()) }
                }
            };

            quote! {
                fn encode(&self) -> (Id, [u8; #name::LEN]) {
                    let mut data = [0u8; #name::LEN];

                    // encode signals here

                    let id = #id_expr;

                    (id, data)
                }
            }
        };

        tokens.extend(quote! {
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
        });
    }
}

struct SignalValueEnum<'a> {
    signal: &'a Signal,
}

impl ToTokens for SignalValueEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let signal = self.signal;

        if signal.value_descriptions.is_empty() {
            return;
        }

        let enum_name = format_ident!("{}", signal.name.0.0.to_upper_camelcase());

        let variants = signal.value_descriptions.iter().map(|vd| {
            let name = format_ident!("{}", vd.description);
            quote! { #name }
        });

        let from_arms = signal.value_descriptions.iter().map(|vd| {
            let name = format_ident!("{}", vd.description);
            let value = vd.value;
            quote! { #value => Self::#name }
        });

        let into_arms = signal.value_descriptions.iter().map(|vd| {
            let name = format_ident!("{}", vd.description);
            let value = vd.value;
            quote! { #enum_name::#name => #value }
        });

        quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum #enum_name {
                #( #variants, )*
                _Other(u8),
            }

            impl From<u8> for #enum_name {
                fn from(val: u8) -> Self {
                    match val {
                        #( #from_arms, )*
                        _ => Self::_Other(val),
                    }
                }
            }

            impl From<#enum_name> for u8 {
                fn from(val: #enum_name) -> Self {
                    match val {
                        #( #into_arms, )*
                        #enum_name::_Other(v) => v,
                    }
                }
            }
        }.to_tokens(tokens);
    }
}
