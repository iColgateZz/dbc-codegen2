use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse2;

use crate::ir::message::{Message, MessageId};

pub struct RustGen;

impl RustGen {
    pub fn generate(messages: &[Message]) -> String {
        let imports = quote! {
            use embedded_can::{Frame, Id, StandardId, ExtendedId};
        };
        let error_enum = Self::error_enum();
        let msg_trait = Self::msg_trait();
        let msg_enum = Self::msg_enum(messages);
        let msg_impl = Self::msg_impl(messages);

        let message_defs: Vec<_> =
            messages.iter().map(Self::message).collect();

        let tokens = quote! {
            #imports

            #error_enum

            #msg_trait

            #msg_enum

            #msg_impl

            #( #message_defs )*
        };

        let file = parse2(tokens).unwrap();

        prettyplease::unparse(&file)
    }

    fn error_enum() -> TokenStream {
        quote! {
            #[derive(Debug, Clone)]
            pub enum CanError {
                Err1,
                Err2,
            }
        }
    }

    fn msg_trait() -> TokenStream {
        quote! {
            pub trait CanMessage<const LEN: usize>: Sized {
                fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError>;
                fn encode(&self) -> (Id, [u8; LEN]);
            }
        }
    }

    fn msg_enum(messages: &[Message]) -> TokenStream {
        let variants = messages.iter().map(|msg| {
            let name = format_ident!("{}", msg.name.0);

            quote! {
                #name(#name)
            }
        });

        quote! {
            #[derive(Debug, Clone)]
            pub enum Msg {
                #( #variants, )*
            }
        }
    }

    fn msg_impl(messages: &[Message]) -> TokenStream {
        let arms = messages.iter().map(|msg| {
            let name = format_ident!("{}", msg.name.0);

            quote! {
                #name::ID => Msg::#name(#name::try_from_frame(frame)?)
            }
        });

        quote! {
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
        }
    }

    fn message(msg: &Message) -> TokenStream {
        let name = format_ident!("{}", msg.name.0);

        let fields = msg.signals.iter().map(|sig| {
            let field = format_ident!("{}", sig.name.0.0);

            quote! {
                pub #field: f64
            }
        });

        let impl_block = Self::message_impl(msg);
        let try_from = Self::try_from_frame(msg);
        let encode = Self::encode(msg);

        quote! {
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
    }

    fn message_impl(msg: &Message) -> TokenStream {
        let name = format_ident!("{}", msg.name.0);

        let id = match msg.id {
            MessageId::Standard(id) => id as u32,
            MessageId::Extended(id) => id,
        };

        let len = msg.size;

        quote! {
            impl #name {
                pub const ID: u32 = #id;
                pub const LEN: usize = #len;
            }
        }
    }

    fn try_from_frame(msg: &Message) -> TokenStream {
        let name = format_ident!("{}", msg.name.0);

        let mut byte = 0usize;

        let reads = msg.signals.iter().map(|sig| {
            let raw = format_ident!("raw_{}", sig.name.0.0);

            let b0 = byte;
            let b1 = byte + 1;
            byte += 2;

            quote! {
                let #raw = u16::from_le_bytes([data[#b0], data[#b1]]);
            }
        });

        let fields = msg.signals.iter().map(|sig| {
            let field = format_ident!("{}", sig.name.0.0);
            let raw = format_ident!("raw_{}", sig.name.0.0);

            let factor = sig.factor;

            quote! {
                #field: #raw as f64 * #factor
            }
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
    }

    fn encode(msg: &Message) -> TokenStream {
        let name = format_ident!("{}", msg.name.0);
        let len = msg.size;

        let id_expr = match msg.id {
            MessageId::Standard(_) => {
                quote! {
                    Id::Standard(StandardId::new(Self::ID as u16).unwrap())
                }
            }
            MessageId::Extended(_) => {
                quote! {
                    Id::Extended(ExtendedId::new(Self::ID).unwrap())
                }
            }
        };

        quote! {
            fn encode(&self) -> (Id, [u8; #name::LEN]) {
                let mut data = [0u8; #len];

                // encode signals here

                let id = #id_expr;

                (id, data)
            }
        }
    }
}
