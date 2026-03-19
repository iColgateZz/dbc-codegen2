use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::File;

use crate::DbcFile;
use crate::ir::message::{Message, MessageId};
use crate::ir::signal::{MultiplexIndicator, Signal};
use crate::ir::signal_layout::{SignalLayout, ByteOrder};
use crate::ir::signal_value_type::{RustIntegerLiteral, RustType, RustFloatLiteral, IntReprType};
use std::collections::BTreeMap;

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

        let signals: Vec<SignalCtx> = msg
            .signal_idxs
            .iter()
            .map(|idx| SignalCtx::new(&self.file.signals[idx.0], self.file))
            .collect();

        //TODO: this should definitely happen on the IR level
        let mut plain = Vec::new();
        let mut muxed: BTreeMap<u64, Vec<&SignalCtx>> = BTreeMap::new();
        let mut mux_signal: Option<&SignalCtx> = None;

        for s in &signals {
            match &s.signal.multiplexer {
                MultiplexIndicator::Plain => plain.push(s),

                MultiplexIndicator::Multiplexor => {
                    mux_signal = Some(s);
                    // plain.push(s);
                }

                MultiplexIndicator::MultiplexedSignal(v) => {
                    muxed.entry(*v).or_default().push(s);
                }

                // intentionally skip
                MultiplexIndicator::MultiplexorAndMultiplexedSignal(_v) => (),
            }
        }

        if muxed.is_empty() {
            self.generate_plain(tokens, &signals);
        } else {
            self.generate_mux(tokens, &signals, plain, muxed, mux_signal.unwrap());
        }
    }
}

impl MessageDef<'_> {
    fn generate_plain(&self, tokens: &mut TokenStream, signals: &Vec<SignalCtx>) {
        let msg = self.msg;
        let name = format_ident!("{}", msg.name.upper_camel());

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

    fn generate_mux(
        &self,
        tokens: &mut TokenStream,
        signals: &Vec<SignalCtx>,
        plain: Vec<&SignalCtx>,
        muxed: BTreeMap<u64, Vec<&SignalCtx>>,
        mux_signal: &SignalCtx,
    ) {
        let msg = self.msg;
        let name = format_ident!("{}", msg.name.upper_camel());
        let mux_enum_name = format_ident!("{}Mux", name);

        let value_enums = signals
            .iter()
            .map(|s| SignalValueEnum { signal: s.signal });

        let id_expr = match msg.id {
            MessageId::Standard(id) => {
                quote! { Id::Standard(unsafe { StandardId::new_unchecked(#id) }) }
            }
            MessageId::Extended(id) => {
                quote! { Id::Extended(unsafe { ExtendedId::new_unchecked(#id) }) }
            }
        };

        let len = msg.size as usize;

        let plain_fields = plain.iter().map(|s| {
            let field = s.field_ident();
            let ty = s.rust_type();
            quote! { pub #field: #ty }
        });

        let plain_reads = plain.iter().map(|s| s.decode_read());
        let plain_init = plain.iter().map(|s| s.decode_field());
        let plain_writes = plain.iter().map(|s| s.encode_write());

        let mux_structs = muxed.iter().map(|(v, sigs)| {
            let struct_name = format_ident!("{}Mux{}", name, v);

            let fields = sigs.iter().map(|s| {
                let field = s.field_ident();
                let ty = s.rust_type();
                quote! { pub #field: #ty }
            });

            let reads = sigs.iter().map(|s| s.decode_read());
            let inits = sigs.iter().map(|s| s.decode_field());
            let writes = sigs.iter().map(|s| s.encode_write());

            quote! {
                #[derive(Debug, Clone)]
                pub struct #struct_name {
                    #( #fields, )*
                }

                impl #struct_name {

                    fn decode_from(data: &[u8]) -> Result<Self, CanError> {

                        #( #reads )*

                        Ok(Self {
                            #( #inits, )*
                        })
                    }

                    fn encode_into(&self, data: &mut [u8]) {
                        #( #writes )*
                    }
                }
            }
        });

        let mux_variants = muxed.keys().map(|v| {
            let struct_name = format_ident!("{}Mux{}", name, v);
            let variant = format_ident!("V{}", v);

            quote! { #variant(#struct_name) }
        });

        let mux_raw_ty = mux_signal.raw_rust_type();
        let (mux_start, mux_end) = mux_signal.start_end_bit();
        let mux_order = mux_signal.bitvec_order();

        let mux_encode_arms = muxed.keys().map(|v| {
            let variant = format_ident!("V{}", v);

            quote! {
                #mux_enum_name::#variant(inner) => {

                    data.view_bits_mut::<#mux_order>()[#mux_start..#mux_end]
                        .store_le(#v as #mux_raw_ty);

                    inner.encode_into(&mut data);
                }
            }
        });

        let mux_read = mux_signal.decode_read();
        let mux_raw = mux_signal.raw_ident();

        let mux_decode_arms = muxed.keys().map(|v| {
            let lit = mux_signal.signal.physical_type.literal(*v as i64);
            let struct_name = format_ident!("{}Mux{}", name, v);
            let variant = format_ident!("V{}", v);

            quote! {
                #lit => #mux_enum_name::#variant(#struct_name::decode_from(data)?)
            }
        });


        let struct_def = quote! {
            #[derive(Debug, Clone)]
            pub struct #name {
                #( #plain_fields, )*
                pub mux: #mux_enum_name,
            }
        };

        let trait_impl = quote! {
            impl CanMessage<{ #name::LEN }> for #name {

                fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {

                    let data = frame.data();

                    if data.len() < Self::LEN {
                        return Err(CanError::InvalidPayloadSize);
                    }

                    #( #plain_reads )*

                    #mux_read

                    let mux = match #mux_raw {

                        #( #mux_decode_arms, )*

                        _ => return Err(CanError::Err2),
                    };

                    Ok(Self {
                        #( #plain_init, )*
                        mux,
                    })
                }

                fn encode(&self) -> [u8; #name::LEN] {

                    let mut data = [0u8; #name::LEN];

                    #( #plain_writes )*

                    match &self.mux {
                        #( #mux_encode_arms )*
                    }

                    data
                }
            }
        };

        let impl_block = quote! {
            impl #name {

                pub const ID: Id = #id_expr;
                pub const LEN: usize = #len;
            }
        };

        quote! {

            #( #value_enums )*

            #( #mux_structs )*

            #[derive(Debug, Clone)]
            pub enum #mux_enum_name {
                #( #mux_variants, )*
            }

            #struct_def

            #impl_block

            #trait_impl
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
            quote! { #value => #enum_name::#name }
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

        let min = self.f64_to_correct_literal_with_type(min);
        let max = self.f64_to_correct_literal_with_type(max);

        //TODO: no need to check if unsigned value is less than 0
        quote! {
            if value < #min || value > #max {
                return Err(CanError::ValueOutOfRange);
            }
        }
    }

    //TODO: move the calculations to a transformer node
    fn start_end_bit(&self) -> (usize, usize) {
        match self.layout.byte_order {
            ByteOrder::LittleEndian => {
                let start = self.layout.start_bit as usize;
                let end = start + self.layout.size as usize;
                (start, end)
            }

            ByteOrder::BigEndian => {
                let start_bit = self.layout.start_bit;

                let x = (start_bit / 8) * 8;
                let y = 7 - (start_bit % 8);

                let start = (x + y) as usize;
                let end = start + self.layout.size as usize;

                (start, end)
            }
        }
    }

    fn bitvec_order(&self) -> TokenStream {
        match self.layout.byte_order {
            ByteOrder::LittleEndian => quote! { Lsb0 },
            ByteOrder::BigEndian => quote! { Msb0 },
        }
    }

    fn decode_read(&self) -> TokenStream {
        let raw = self.raw_ident();
        let (start, end) = self.start_end_bit();
        let order = self.bitvec_order();

        if self.is_enum() || !self.is_float() {
            let raw_ty = self.raw_rust_type();
            quote! { let #raw = data.view_bits::<#order>()[#start..#end].load_le::<#raw_ty>(); }
        } else {
            // bitvec cannot read f32/f64 from bits. Code finds the best fitting unsigned type
            // and reads data into the type. The data is later casted to the correct float type.
            let int_ty = self.int_repr_for_float();
            quote! { let #raw = data.view_bits::<#order>()[#start..#end].load_le::<#int_ty>(); }
        }
    }

    //TODO: add a checker node that ensures that */+- operations
    //      are safe. dbc-codegen uses saturating_*, checked_*
    //TODO: do not perform multiplication when factor is 1
    //      do not perform addition when offset is 0
    fn decode_field(&self) -> TokenStream {
        let field = self.field_ident();
        let raw = self.raw_ident();

        if self.is_enum() {
            let enum_name = self.enum_ident();
            let raw_ty = self.raw_rust_type();
            quote! { #field: #enum_name::from(#raw as #raw_ty) }
        } else if self.is_float() {
            let factor = self.factor_literal();
            let offset = self.offset_literal();
            // bitvec does not work with floats. See comment in decode_read!
            let ty = format_ident!("{}", self.signal.physical_type.as_rust_type());
            quote! { #field: (#raw as #ty) * (#factor) + (#offset) }
        } else {
            let factor = self.factor_literal();
            let offset = self.offset_literal();
            quote! { #field: (#raw) * (#factor) + (#offset) }
        }
    }

    //TODO: do not perform division when factor is 1
    //      do not perform subtraction when offset is 0
    fn encode_write(&self) -> TokenStream {
        let field = self.field_ident();
        let (start, end) = self.start_end_bit();
        let order = self.bitvec_order();

        if self.is_enum() {
            let ty = self.raw_rust_type();
            quote! { data.view_bits_mut::<#order>()[#start..#end].store_le(#ty::from(self.#field)); }
        } else if self.is_float() {
            let factor = self.factor_literal();
            let offset = self.offset_literal();
            // bitvec does not work with floats. See comment in decode_read!
            let int_ty = self.int_repr_for_float();
            quote! {
                data.view_bits_mut::<#order>()[#start..#end].store_le(((self.#field - (#offset)) / (#factor)) as #int_ty);
            }
        } else {
            let factor = self.factor_literal();
            let offset = self.offset_literal();
            quote! {
                data.view_bits_mut::<#order>()[#start..#end].store_le((self.#field - (#offset)) / (#factor));
            }
        }
    }

}
