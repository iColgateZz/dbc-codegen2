use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::File;

use crate::DbcFile;
use crate::ir::message::{Message, MessageId};
use crate::ir::signal::Signal;
use crate::ir::signal_value_type::PhysicalType;
use crate::ir::signal_value_type::{RustLiteral, RustType};

pub struct RustGen;

impl RustGen {
    pub fn generate(file: &DbcFile) -> String {
        let imports = quote! {
            use embedded_can::{Frame, Id, StandardId, ExtendedId};
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
                fn encode(&self) -> (Id, [u8; LEN]);
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

        let signals: Vec<&Signal> = msg.signal_idxs.iter().map(|idx| &self.file.signals[idx.0]).collect();

        let value_enums = signals.iter().map(|s| SignalValueEnum { signal: s });

        let fields = signals.iter().map(|sig| {
            let field = format_ident!("{}", sig.name.lower());
            let rust_type = sig
                .signal_value_enum
                .as_ref()
                .map(|_| format_ident!("{}", sig.name.upper_camel()))
                .unwrap_or(format_ident!("{}", sig.physical_type.as_rust_type()));
            quote! { pub #field: #rust_type }
        });

        let id = match msg.id {
            MessageId::Standard(id) => id as u32,
            MessageId::Extended(id) => id,
        };
        let len = msg.size as usize;

        let constructor_params = signals.iter().map(|sig| {
            let field = format_ident!("{}", sig.name.lower());
            let rust_type = sig
                .signal_value_enum
                .as_ref()
                .map(|_| format_ident!("{}", sig.name.upper_camel()))
                .unwrap_or(format_ident!("{}", sig.physical_type.as_rust_type()));

            quote! { #field: #rust_type }
        });

        let constructor_fields: Vec<_> = signals
            .iter()
            .map(|sig| format_ident!("{}", sig.name.lower()))
            .collect();

        let constructor_validations = constructor_fields.iter().map(|field| {
            let setter = format_ident!("set_{}", field);
            quote! {
                msg.#setter(msg.#field)?;
            }
        });

        let getters = signals.iter().map(|sig| {
            let field = format_ident!("{}", sig.name.lower());
            let rust_type = sig
                .signal_value_enum
                .as_ref()
                .map(|_| format_ident!("{}", sig.name.upper_camel()))
                .unwrap_or(format_ident!("{}", sig.physical_type.as_rust_type()));

            quote! {
                pub fn #field(&self) -> #rust_type {
                    self.#field
                }
            }
        });

        let setters = signals.iter().map(|sig| {
            let field = format_ident!("{}", sig.name.lower());
            let set_name = format_ident!("set_{}", sig.name.lower());

            let rust_type = sig
                .signal_value_enum
                .as_ref()
                .map(|_| format_ident!("{}", sig.name.upper_camel()))
                .unwrap_or(format_ident!("{}", sig.physical_type.as_rust_type()));

            let sig_layout = self.file.signal_layouts[sig.layout.0];
            let check = if sig.signal_value_enum.is_some() {
                quote! {}
            } else {
                let phys = &sig.physical_type;

                let min_literal = match phys {
                    PhysicalType::Float32 => {
                        let v = sig_layout.min as f32;
                        quote! { #v }
                    }
                    PhysicalType::Float64 => {
                        let v = sig_layout.min as f64;
                        quote! { #v }
                    }
                    PhysicalType::Integer(_) => {
                        phys.literal(sig_layout.min as i64).to_token_stream()
                    }
                    PhysicalType::Enum { .. } => {
                        quote! {}
                    }
                };

                let max_literal = match phys {
                    PhysicalType::Float32 => {
                        let v = sig_layout.max as f32;
                        quote! { #v }
                    }
                    PhysicalType::Float64 => {
                        let v = sig_layout.max as f64;
                        quote! { #v }
                    }
                    PhysicalType::Integer(_) => {
                        phys.literal(sig_layout.max as i64).to_token_stream()
                    }
                    PhysicalType::Enum { .. } => {
                        quote! {}
                    }
                };

                quote! {
                    if value < #min_literal || value > #max_literal {
                        return Err(CanError::ValueOutOfRange);
                    }
                }
            };

            quote! {
                pub fn #set_name(&mut self, value: #rust_type) -> Result<(), CanError> {
                    #check
                    self.#field = value;
                    Ok(())
                }
            }
        });

        let impl_block = quote! {
            impl #name {
                pub const ID: u32 = #id;
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
            // TODO: take into account the multiplexing, otherwise
            //       conflicting reads happen (signals are in different
            //       mux groups - they share same bit positions)
            let reads = signals.iter().map(|sig| {
                let raw = format_ident!("raw_{}", sig.name.lower());
                let sig_layout = self.file.signal_layouts[sig.layout.0];
                let byte_count = sig_layout.size.div_ceil(8) as usize;
                let start_byte = sig_layout.start_bit as usize / 8;
                let indices: Vec<usize> = (start_byte..start_byte + byte_count).collect();

                match byte_count {
                    1 => quote! { let #raw = data[#(#indices)*]; },
                    2 => quote! { let #raw = u16::from_le_bytes([#(data[#indices]),*]); },
                    4 => quote! { let #raw = u32::from_le_bytes([#(data[#indices]),*]); },
                    _ => panic!("unsupported signal size: {} bits", sig_layout.size),
                }
            });

            let fields = signals.iter().map(|sig| {
                let field = format_ident!("{}", sig.name.lower());
                let raw = format_ident!("raw_{}", sig.name.lower());
                let sig_layout = self.file.signal_layouts[sig.layout.0];
                let factor = sig_layout.factor;

                let value = if let Some(_sve) = &sig.signal_value_enum {
                    let enum_name = format_ident!("{}", sig.name.upper_camel());
                    let rust_type = format_ident!("{}", sig.physical_type.as_rust_type());
                    quote! { #enum_name::from(#raw as #rust_type) }
                } else {
                    quote! { #raw as f64 * #factor }
                };

                quote! { #field: #value }
            });

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
            let id_expr = match msg.id {
                MessageId::Standard(_) => {
                    quote! { Id::Standard(StandardId::new(Self::ID as u16).unwrap()) }
                }
                MessageId::Extended(_) => {
                    quote! { Id::Extended(ExtendedId::new(Self::ID).unwrap()) }
                }
            };

            // TODO: take into account the multiplexing, otherwise
            //       data overwriting happens (signals are in different
            //       mux groups - they share same bit positions)
            let writes = signals.iter().map(|sig| {
                let sig_layout = self.file.signal_layouts[sig.layout.0];
                let field = format_ident!("{}", sig.name.lower());
                let byte_count = sig_layout.size.div_ceil(8) as usize;
                let start_byte = sig_layout.start_bit as usize / 8;
                let indices: Vec<usize> = (start_byte..start_byte + byte_count).collect();
                let slot_indices: Vec<usize> = (0..byte_count).collect();

                let raw_value = if let Some(_sve) = &sig.signal_value_enum {
                    let rust_type = format_ident!("{}", sig.physical_type.as_rust_type());
                    quote! { #rust_type::from(self.#field) }
                } else {
                    let factor = sig_layout.factor;
                    match byte_count {
                        1 => quote! { (self.#field / #factor) as u8 },
                        2 => quote! { (self.#field / #factor) as u16 },
                        4 => quote! { (self.#field / #factor) as u32 },
                        _ => panic!("unsupported signal size: {} bits", sig_layout.size),
                    }
                };

                match byte_count {
                    1 => {
                        let idx = indices[0];
                        quote! { data[#idx] = #raw_value; }
                    }
                    2 => quote! {
                        let bytes = (#raw_value).to_le_bytes();
                        #( data[#indices] = bytes[#slot_indices]; )*
                    },
                    4 => quote! {
                        let bytes = (#raw_value).to_le_bytes();
                        #( data[#indices] = bytes[#slot_indices]; )*
                    },
                    _ => panic!("unsupported signal size: {} bits", sig_layout.size),
                }
            });

            quote! {
                fn encode(&self) -> (Id, [u8; #name::LEN]) {
                    let mut data = [0u8; #name::LEN];

                    #( #writes )*

                    let id = #id_expr;
                    (id, data)
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
