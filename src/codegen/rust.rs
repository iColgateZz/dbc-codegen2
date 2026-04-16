use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::File;

use crate::DbcFile;
use crate::codegen::config::CodegenConfig;
use crate::ir::message::{Message, MessageId, MessageSignalClassification};
use crate::ir::signal::{Receiver, Signal};
use crate::ir::signal_layout::{ByteOrder, SignalLayout};
use crate::ir::signal_value_enum::SignalValueEnum;
use crate::ir::signal_value_type::{IntReprType, RustFloatLiteral, RustIntegerLiteral, RustType};
use heck::ToUpperCamelCase;
use std::collections::BTreeMap;

pub struct RustGen;

impl RustGen {
    pub fn generate(file: &DbcFile, config: &CodegenConfig) -> String {
        let imports = quote! {
            use embedded_can::{Frame, Id, StandardId, ExtendedId};
            use bitvec::prelude::*;
            use core::ops::BitOr;
        };

        let messages = &file.messages;
        let value_enums = file.signal_value_enums.iter().map(|e| SignalValueEnumCtx {
            enum_def: e,
            config,
        });
        let error_enum = ErrorEnum;
        let msg_trait = MsgTrait;
        let msg_enum = MsgEnum { messages };
        let message_defs: Vec<_> = messages
            .iter()
            .map(|m| MessageDef {
                msg: m,
                file,
                config: config,
            })
            .collect();

        let tokens = quote! {
            #imports

            #error_enum

            #( #value_enums )*

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
                UnknownFrameId,
                UnknownMuxValue,
                InvalidPayloadSize,
                ValueOutOfRange,
                IvalidEnumValue,
            }
        }
        .to_tokens(tokens);
    }
}

struct MsgTrait;

impl ToTokens for MsgTrait {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote! {
            pub trait CanMessageTrait<const LEN: usize>: Sized {
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
                        _ => return Err(CanError::UnknownFrameId),
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
    config: &'a CodegenConfig,
}

impl ToTokens for MessageDef<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.msg.classify_signals(&self.file.signals) {
            MessageSignalClassification::Plain { signals } => {
                let ctxs: Vec<SignalCtx> = signals
                    .iter()
                    .map(|idx| SignalCtx::new(&self.file.signals[idx.0], self.file, self.config))
                    .collect();
                self.generate_plain(tokens, &ctxs);
            }
            MessageSignalClassification::Multiplexed {
                plain: _,
                mux_signal,
                muxed,
            } => {
                let all_ctxs: Vec<SignalCtx> = self
                    .msg
                    .signal_idxs
                    .iter()
                    .map(|idx| SignalCtx::new(&self.file.signals[idx.0], self.file, self.config))
                    .collect();
                let mux_ctx = &all_ctxs[self
                    .msg
                    .signal_idxs
                    .iter()
                    .position(|i| i == &mux_signal)
                    .unwrap()];
                let muxed_ctxs: BTreeMap<u64, Vec<&SignalCtx>> = muxed
                    .iter()
                    .map(|(v, idxs)| {
                        let sigs = idxs
                            .iter()
                            .map(|idx| {
                                &all_ctxs
                                    [self.msg.signal_idxs.iter().position(|i| i == idx).unwrap()]
                            })
                            .collect();
                        (*v, sigs)
                    })
                    .collect();

                // should refactor generate_mux to take SignalIdx values directly
                // and build SignalCtx internally
                // would make it simpler here
                self.generate_mux(tokens, muxed_ctxs, mux_ctx);
            }
        }
    }
}

impl MessageDef<'_> {
    fn generate_plain(&self, tokens: &mut TokenStream, signals: &Vec<SignalCtx>) {
        let msg = self.msg;
        let name = format_ident!("{}", msg.name.upper_camel());
        let signals: Vec<&SignalCtx> = signals.iter().collect();

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

        let constructor_params = Self::gen_constructor_params(&signals);
        let constructor_body = Self::gen_constructor_body(&signals);
        let getters = Self::gen_getters(&signals);
        let setters = Self::gen_setters(&signals);

        let doc = message_doc(&msg);

        quote! {
            #doc
            #[derive(Debug, Clone)]
            pub struct #name {
                data: [u8; #len],
            }

            impl #name {
                pub const ID: Id = #id_expr;
                pub const LEN: usize = #len;

                pub fn new(
                    #( #constructor_params ),*
                ) -> Result<Self, CanError> {
                    let mut msg = Self {
                        data: [0u8; Self::LEN],
                    };

                    #constructor_body

                    Ok(msg)
                }

                #( #getters )*

                #( #setters )*
            }

            impl CanMessageTrait<{ Self::LEN }> for #name {

                fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
                    let data = frame.data();

                    if data.len() < Self::LEN {
                        return Err(CanError::InvalidPayloadSize);
                    }

                    let mut buf = [0u8; #len];
                    buf.copy_from_slice(&data[..#len]);

                    Ok(Self { data: buf })
                }

                fn encode(&self) -> [u8; Self::LEN] {
                    self.data
                }
            }
        }
        .to_tokens(tokens);
    }

    fn gen_constructor_body(signals: &[&SignalCtx]) -> TokenStream {
        let setters = signals.iter().map(|s| {
            let field = s.field_ident();
            let setter = s.setter_ident();

            quote! {
                msg.#setter(#field)?;
            }
        });

        quote! {
            #( #setters )*
        }
    }

    fn generate_mux(
        &self,
        tokens: &mut TokenStream,
        muxed: BTreeMap<u64, Vec<&SignalCtx>>,
        mux_signal: &SignalCtx,
    ) {
        let msg = self.msg;
        let name = format_ident!("{}", msg.name.upper_camel());
        let mux_enum = format_ident!("{}Mux", name);

        let id_expr = match msg.id {
            MessageId::Standard(id) => {
                quote! { Id::Standard(unsafe { StandardId::new_unchecked(#id) }) }
            }
            MessageId::Extended(id) => {
                quote! { Id::Extended(unsafe { ExtendedId::new_unchecked(#id) }) }
            }
        };

        let len = msg.size as usize;

        let doc = message_doc(&msg);

        let variant_structs = muxed.iter().map(|(idx, sigs)| {
            let struct_name = format_ident!("{}Mux{}", name, idx);

            let ctxs: Vec<&SignalCtx> = sigs.iter().copied().collect();
            let getters = Self::gen_getters(&ctxs);
            let setters = Self::gen_setters(&ctxs);

            quote! {
                #[derive(Debug, Clone, Default)]
                pub struct #struct_name {
                    data: [u8; #len],
                }

                impl #struct_name {
                    pub fn new() -> Self {
                        Self { data: [0u8; #len] }
                    }

                    #( #getters )*
                    #( #setters )*
                }
            }
        });

        let mux_variants = muxed.keys().map(|idx| {
            let variant = format_ident!("V{}", idx);
            let struct_name = format_ident!("{}Mux{}", name, idx);

            quote! { #variant(#struct_name) }
        });

        let mux_read = mux_signal.decode_read();
        let raw_ident = mux_signal.raw_ident();

        let mux_match_arms = muxed.keys().map(|idx| {
            let variant = format_ident!("V{}", idx);
            let struct_name = format_ident!("{}Mux{}", name, idx);
            let lit = syn::LitInt::new(&format!("{}", idx), Span::call_site());

            quote! {
                #lit => Ok(#mux_enum::#variant(#struct_name { data: self.data }))
            }
        });

        let mux_getter = quote! {
            pub fn mux(&self) -> Result<#mux_enum, CanError> {
                #mux_read

                match #raw_ident {
                    #( #mux_match_arms, )*
                    _ => Err(CanError::UnknownMuxValue),
                }
            }
        };

        let mux_setters = muxed.keys().map(|idx| {
            let fn_name = format_ident!("set_mux_{}", idx);
            let struct_name = format_ident!("{}Mux{}", name, idx);

            let mux_raw_ty = mux_signal.raw_rust_type();

            let (start, end) = mux_signal.start_end_bit();
            let order = mux_signal.bitvec_order();
            let store = mux_signal.store_fn();

            quote! {
                pub fn #fn_name(&mut self, value: #struct_name) -> Result<(), CanError> {
                    let b0 = BitArray::<_, LocalBits>::new(self.data);
                    let b1 = BitArray::<_, LocalBits>::new(value.data);

                    self.data = b0.bitor(b1).into_inner();

                    self.data.view_bits_mut::<#order>()[#start..#end]
                        .#store(#idx as #mux_raw_ty);

                    Ok(())
                }
            }
        });

        quote! {
            #[derive(Debug, Clone)]
            pub enum #mux_enum {
                #( #mux_variants, )*
            }

            #( #variant_structs )*

            #doc
            #[derive(Debug, Clone)]
            pub struct #name {
                data: [u8; #len],
            }

            impl #name {
                pub const ID: Id = #id_expr;
                pub const LEN: usize = #len;

                #mux_getter

                #( #mux_setters )*
            }

            impl CanMessageTrait<{ Self::LEN }> for #name {
                fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
                    let data = frame.data();

                    if data.len() < Self::LEN {
                        return Err(CanError::InvalidPayloadSize);
                    }

                    let mut buf = [0u8; #len];
                    buf.copy_from_slice(&data[..#len]);

                    Ok(Self { data: buf })
                }

                fn encode(&self) -> [u8; #len] {
                    self.data
                }
            }
        }
        .to_tokens(tokens);
    }

    fn gen_constructor_params(signals: &[&SignalCtx]) -> impl Iterator<Item = TokenStream> {
        signals.iter().map(|s| {
            let field = s.field_ident();
            let ty = s.rust_type();
            quote! { #field: #ty }
        })
    }

    fn gen_getters(signals: &[&SignalCtx]) -> impl Iterator<Item = TokenStream> {
        signals.iter().map(|s| {
            let field = s.field_ident();
            let ty = s.rust_type();
            let doc = signal_doc(&s);

            let read = s.decode_read();
            let expr = s.decode_expr();

            if s.is_enum() && s.config.no_enum_other {
                quote! {
                    #doc
                    pub fn #field(&self) -> Result<#ty, CanError> {
                        #read

                        Ok(#expr)
                    }
                }
            } else {
                quote! {
                    #doc
                    pub fn #field(&self) -> #ty {
                        #read

                        #expr
                    }
                }
            }
        })
    }

    fn gen_setters(signals: &[&SignalCtx]) -> impl Iterator<Item = TokenStream> {
        signals.iter().map(|s| {
            let setter = s.setter_ident();
            let ty = s.rust_type();
            let check = s.range_check();
            let write = s.encode_write();

            quote! {
                pub fn #setter(&mut self, value: #ty) -> Result<(), CanError> {
                    #check

                    #write

                    Ok(())
                }
            }
        })
    }
}

struct SignalValueEnumCtx<'a> {
    enum_def: &'a SignalValueEnum,
    config: &'a CodegenConfig,
}

impl ToTokens for SignalValueEnumCtx<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let enum_def = self.enum_def;

        let enum_name = format_ident!("{}", self.enum_def.name.to_upper_camel_case());
        let repr_type = &enum_def.phys_type;
        let rust_type = format_ident!("{}", repr_type.as_rust_type());

        let enum_def_tokens = if !self.config.no_enum_other {
            self.gen_with_other(&enum_name, &rust_type, enum_def)
        } else {
            self.gen_without_other(&enum_name, &rust_type, enum_def)
        };

        enum_def_tokens.to_tokens(tokens);
    }
}

impl<'a> SignalValueEnumCtx<'a> {
    fn gen_with_other(
        &self,
        enum_name: &Ident,
        rust_type: &Ident,
        sve: &SignalValueEnum,
    ) -> TokenStream {
        let variants = Self::gen_enum_variants(sve);
        let from_arms = self.gen_from_arms(sve);
        let into_arms = self.gen_into_arms(sve);

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
                        _ => #enum_name::_Other(val),
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
    }

    fn gen_without_other(
        &self,
        enum_name: &Ident,
        rust_type: &Ident,
        sve: &SignalValueEnum,
    ) -> TokenStream {
        let variants = Self::gen_enum_variants(sve);
        let from_arms = self.gen_from_arms(sve);
        let into_arms = self.gen_into_arms(sve);

        quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum #enum_name {
                #( #variants, )*
            }

            impl TryFrom<#rust_type> for #enum_name {
                type Error = CanError;

                fn try_from(val: #rust_type) -> Result<Self, CanError> {
                    Ok(match val {
                        #( #from_arms, )*
                        _ => return Err(CanError::IvalidEnumValue),
                    })
                }
            }

            impl From<#enum_name> for #rust_type {
                fn from(val: #enum_name) -> Self {
                    match val {
                        #( #into_arms, )*
                    }
                }
            }
        }
    }

    fn gen_enum_variants(sve: &SignalValueEnum) -> impl Iterator<Item = TokenStream> {
        sve.variants.iter().map(|vd| {
            let name = format_ident!("{}", vd.description);
            quote! { #name }
        })
    }

    fn gen_from_arms(&self, sve: &SignalValueEnum) -> impl Iterator<Item = TokenStream> {
        let enum_name = format_ident!("{}", self.enum_def.name.to_upper_camel_case());
        let repr_type = &self.enum_def.phys_type;

        sve.variants.iter().map(move |vd| {
            let name = format_ident!("{}", vd.description);
            let value = repr_type.literal(vd.value);
            quote! { #value => #enum_name::#name }
        })
    }

    fn gen_into_arms(&self, sve: &SignalValueEnum) -> impl Iterator<Item = TokenStream> {
        let enum_name = format_ident!("{}", self.enum_def.name.to_upper_camel_case());
        let repr_type = &self.enum_def.phys_type;

        sve.variants.iter().map(move |vd| {
            let name = format_ident!("{}", vd.description);
            let value = repr_type.literal(vd.value);
            quote! { #enum_name::#name => #value }
        })
    }
}

struct SignalCtx<'a> {
    signal: &'a Signal,
    layout: &'a SignalLayout,
    sve: Option<&'a SignalValueEnum>,
    config: &'a CodegenConfig,
}

impl<'a> SignalCtx<'a> {
    fn new(signal: &'a Signal, file: &'a DbcFile, config: &'a CodegenConfig) -> Self {
        let sve = signal
            .signal_value_enum_idx
            .map(|idx| &file.signal_value_enums[idx.0]);

        Self {
            signal,
            layout: &file.signal_layouts[signal.layout.0],
            sve,
            config,
        }
    }

    fn field_ident(&self) -> syn::Ident {
        format_ident!("{}", self.signal.name.snake_case())
    }

    fn setter_ident(&self) -> syn::Ident {
        format_ident!("set_{}", self.signal.name.snake_case())
    }

    fn raw_ident(&self) -> syn::Ident {
        format_ident!("raw_{}", self.signal.name.snake_case())
    }

    fn enum_ident(&self) -> syn::Ident {
        let sve = self.sve.expect("enum_ident called without enum");
        format_ident!("{}", sve.name.to_upper_camel_case())
    }

    fn rust_type(&self) -> syn::Ident {
        if self.sve.is_some() {
            self.enum_ident()
        } else {
            format_ident!("{}", self.signal.physical_type.as_rust_type())
        }
    }

    fn raw_rust_type(&self) -> syn::Ident {
        format_ident!("{}", self.signal.physical_type.as_rust_type())
    }

    fn is_enum(&self) -> bool {
        self.signal.signal_value_enum_idx.is_some()
    }

    fn is_float(&self) -> bool {
        self.signal.physical_type.is_float()
    }

    fn factor_literal(&self) -> TokenStream {
        let phys = &self.signal.physical_type;
        if self.is_float() {
            phys.fliteral(self.layout.factor).to_token_stream()
        } else {
            phys.literal(self.layout.factor as i64).to_token_stream()
        }
    }

    fn offset_literal(&self) -> TokenStream {
        let phys = &self.signal.physical_type;
        if self.is_float() {
            phys.fliteral(self.layout.offset).to_token_stream()
        } else {
            phys.literal(self.layout.offset as i64).to_token_stream()
        }
    }

    fn int_repr_for_float(&self) -> syn::Ident {
        let ty = IntReprType::from_size_sign(self.layout.size, false);
        format_ident!("{}", ty.as_rust_type())
    }

    fn f64_to_correct_literal_with_type(&self, value: f64) -> TokenStream {
        let phys = &self.signal.physical_type;
        if self.is_float() {
            phys.fliteral(value).to_token_stream()
        } else {
            phys.literal(value as i64).to_token_stream()
        }
    }

    fn range_check(&self) -> TokenStream {
        if self.is_enum() {
            return quote! {};
        }

        let min = self.layout.min;
        let max = self.layout.max;

        if self.config.zero_zero_range_allows_all && min == max && min == 0.0 {
            return quote! {};
        }

        let min = self.f64_to_correct_literal_with_type(min);
        let max = self.f64_to_correct_literal_with_type(max);

        //TODO: no need to check if unsigned value is less than 0
        quote! {
            if value < #min || value > #max {
                return Err(CanError::ValueOutOfRange);
            }
        }
    }

    fn start_end_bit(&self) -> (usize, usize) {
        (self.layout.bitvec_start, self.layout.bitvec_end)
    }

    fn bitvec_order(&self) -> TokenStream {
        match self.layout.byte_order {
            ByteOrder::LittleEndian => quote! { Lsb0 },
            ByteOrder::BigEndian => quote! { Msb0 },
        }
    }

    fn load_fn(&self) -> syn::Ident {
        match self.layout.byte_order {
            ByteOrder::LittleEndian => format_ident!("load_le"),
            ByteOrder::BigEndian => format_ident!("load_be"),
        }
    }

    fn store_fn(&self) -> syn::Ident {
        match self.layout.byte_order {
            ByteOrder::LittleEndian => format_ident!("store_le"),
            ByteOrder::BigEndian => format_ident!("store_be"),
        }
    }

    fn decode_read(&self) -> TokenStream {
        let raw = self.raw_ident();
        let (start, end) = self.start_end_bit();
        let order = self.bitvec_order();
        let load = self.load_fn();

        if self.is_enum() || !self.is_float() {
            let raw_ty = self.raw_rust_type();
            quote! { let #raw = self.data.view_bits::<#order>()[#start..#end].#load::<#raw_ty>(); }
        } else {
            // bitvec cannot read f32/f64 from bits. Code finds the best fitting unsigned type
            // and reads data into the type. The data is later casted to the correct float type.
            let int_ty = self.int_repr_for_float();
            quote! { let #raw = self.data.view_bits::<#order>()[#start..#end].#load::<#int_ty>(); }
        }
    }

    //TODO: add a checker node that ensures that */+- operations
    //      are safe. dbc-codegen uses saturating_*, checked_*
    //TODO: do not perform multiplication when factor is 1
    //      do not perform addition when offset is 0
    fn decode_expr(&self) -> TokenStream {
        let raw = self.raw_ident();

        if self.is_enum() {
            let enum_name = self.enum_ident();
            let raw_ty = self.raw_rust_type();

            if self.config.no_enum_other {
                quote! { #enum_name::try_from(#raw as #raw_ty)? }
            } else {
                quote! { #enum_name::from(#raw as #raw_ty) }
            }
        } else if self.is_float() {
            let factor = self.factor_literal();
            let offset = self.offset_literal();
            let ty = format_ident!("{}", self.signal.physical_type.as_rust_type());
            quote! { (#raw as #ty) * (#factor) + (#offset) }
        } else {
            let factor = self.factor_literal();
            let offset = self.offset_literal();
            quote! { (#raw) * (#factor) + (#offset) }
        }
    }

    //TODO: do not perform division when factor is 1
    //      do not perform subtraction when offset is 0
    fn encode_write(&self) -> TokenStream {
        let (start, end) = self.start_end_bit();
        let order = self.bitvec_order();
        let store = self.store_fn();

        if self.is_enum() {
            let ty = self.raw_rust_type();
            quote! { self.data.view_bits_mut::<#order>()[#start..#end].#store(#ty::from(value)); }
        } else if self.is_float() {
            let factor = self.factor_literal();
            let offset = self.offset_literal();
            // bitvec does not work with floats. See comment in decode_read!
            let int_ty = self.int_repr_for_float();
            quote! {
                self.data.view_bits_mut::<#order>()[#start..#end].#store(((value - (#offset)) / (#factor)) as #int_ty);
            }
        } else {
            let factor = self.factor_literal();
            let offset = self.offset_literal();
            quote! {
                self.data.view_bits_mut::<#order>()[#start..#end].#store((value - (#offset)) / (#factor));
            }
        }
    }
}

fn message_doc(msg: &Message) -> TokenStream {
    let name = &msg.name.raw();

    let id_text = match msg.id {
        MessageId::Standard(id) => {
            format!("Standard {} (0x{:X})", id, id)
        },
        MessageId::Extended(id) => {
            format!("Extended {} (0x{:X})", id, id)
        }
    };

    let size = msg.size;
    let transmitter = match &msg.transmitter {
        crate::ir::message::Transmitter::Node(name) => name.raw(),
        crate::ir::message::Transmitter::VectorXXX => "VectorXXX"
    };

    let mut lines = vec![
        format!("{}", name),
        format!("- ID: {}", id_text),
        format!("- Size: {} bytes", size),
        format!("- Transmitter: {}", transmitter),
    ];

    if let Some(comment) = &msg.comment {
        lines.push("".into());
        lines.extend(comment.lines().map(|l| l.to_string()));
    }

    quote! {
        #( #[doc = #lines] )*
    }
}

fn signal_doc(sig: &SignalCtx) -> TokenStream {
    let s = sig.signal;
    let layout = sig.layout;

    let name = &s.name;
    let min = layout.min;
    let max = layout.max;
    let unit = &s.unit;
    let receivers = if s.receivers.is_empty() {
        "".into()
    } else {
        s.receivers
        .iter()
        .map(|r| match r {
            Receiver::Node(id) => id.raw().to_string(),
            Receiver::VectorXXX => "VectorXXX".to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
    };

    let (start, _) = sig.start_end_bit();
    let size = layout.size;
    let factor = layout.factor;
    let offset = layout.offset;

    let byte_order = match layout.byte_order {
        ByteOrder::LittleEndian => "LittleEndian",
        ByteOrder::BigEndian => "BigEndian",
    };

    let signed = match &layout.value_type {
        crate::ir::signal_layout::ValueType::Unsigned => "unsigned",
        crate::ir::signal_layout::ValueType::Signed => "signed",
    };

    let mut lines = vec![
        format!("{}", name.raw()),
        format!("- Min: {}", min),
        format!("- Max: {}", max),
        format!("- Unit: {}", unit),
        format!("- Receivers: {}", receivers),
        format!("- Start bit: {}", start),
        format!("- Size: {} bits", size),
        format!("- Factor: {}", factor),
        format!("- Offset: {}", offset),
        format!("- Byte order: {}", byte_order),
        format!("- Type: {}", signed),
    ];

    if let Some(comment) = &s.comment {
        lines.push("".into());
        lines.extend(comment.lines().map(|l| l.to_string()));
    }

    quote! {
        #( #[doc = #lines] )*
    }
}
