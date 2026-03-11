use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::File;

use crate::ir::message::{Message, MessageId};

pub struct RustGen;

impl RustGen {
    pub fn generate(messages: &[Message]) -> String {
        let imports = quote! {
            use embedded_can::{Frame, Id, StandardId, ExtendedId};
        };

        let error_enum = ErrorEnum;
        let msg_trait = MsgTrait;
        let msg_enum = MsgEnum { messages };
        let msg_impl = MsgEnumImpl { messages };
        let message_defs: Vec<_> = messages.iter().map(|m| MessageDef { msg: m }).collect();

        let tokens = quote! {
            #imports

            #error_enum

            #msg_trait

            #msg_enum

            #msg_impl

            #( #message_defs )*
        };

        let file: File = syn::parse2(tokens).unwrap();
        println!("{}", file.attrs.len());
        println!("{}", file.items.len());
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
    }
}

struct MsgEnumImpl<'a> {
    messages: &'a [Message],
}

impl ToTokens for MsgEnumImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
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