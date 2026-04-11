use std::collections::BTreeMap;

use heck::ToSnakeCase;

use crate::{
    DbcFile,
    codegen::Generator,
    empty, end_block,
    ir::{
        message::{Message, MessageId, MessageSignalClassification},
        signal::Signal,
        signal_layout::ByteOrder,
        signal_value_enum::SignalValueEnum,
        signal_value_type::CppType,
    },
    line, start_block,
};

pub struct CppGen;

impl CppGen {
    pub fn generate(file: &DbcFile) -> String {
        let mut out = Generator::new();

        line!(out, "#pragma once");
        empty!(out);

        Self::includes(&mut out);
        Self::errors(&mut out);
        Self::endian_read_and_write(&mut out);

        for message in &file.messages {
            Self::message(&mut out, message, file);
        }

        out.into_string()
    }

    fn includes(out: &mut Generator) {
        const INCLUDES: &[&str] = &[
            "array", "bit", "cstddef", "cstdint", "cstdio", "cstring", "expected", "span",
            "variant",
        ];

        for include in INCLUDES {
            line!(out, "#include <{}>", include);
        }
        empty!(out);
    }

    fn errors(out: &mut Generator) {
        const ERRORS: &[&str] = &["UnknownId", "InvalidLength", "InvalidData"];

        start_block!(out, "enum class CanError : uint8_t");
        for error in ERRORS {
            line!(out, "{},", error);
        }
        end_block!(out, "");
        empty!(out);
    }

    fn endian_read_fn(out: &mut Generator, name: &str, swap_if: &str) {
        line!(out, "template <typename T>");
        start_block!(
            out,
            "[[nodiscard]] constexpr T {name}(const uint8_t *d) noexcept"
        );
        line!(out, "T v{{}};");
        line!(out, "std::memcpy(&v, d, sizeof(T));");
        line!(
            out,
            "if constexpr (std::endian::native == std::endian::{swap_if}) v = std::byteswap(v);"
        );
        end_block!(out, "return v;");
        empty!(out);
    }

    fn endian_write_fn(out: &mut Generator, name: &str, swap_if: &str) {
        line!(out, "template <typename T>");
        start_block!(out, "constexpr void {name}(uint8_t *d, T v) noexcept");
        line!(
            out,
            "if constexpr (std::endian::native == std::endian::{swap_if}) v = std::byteswap(v);"
        );
        end_block!(out, "std::memcpy(d, &v, sizeof(T));");
        empty!(out);
    }

    fn extract_le_fn(out: &mut Generator) {
        line!(out, "template <typename T>");
        start_block!(
            out,
            "[[nodiscard]] constexpr T extract_le(const uint8_t* data, std::size_t start, std::size_t end) noexcept"
        );
        line!(out, "using U = std::make_unsigned_t<T>;");
        line!(out, "U result = 0;");
        line!(out, "const std::size_t len = end - start;");
        start_block!(out, "for (std::size_t i = 0; i < len; ++i)");
        line!(out, "const std::size_t bit_idx = start + i;");
        line!(
            out,
            "result |= static_cast<U>((data[bit_idx / 8] >> (bit_idx % 8)) & 0x1u) << i;"
        );
        end_block!(out, "");
        start_block!(out, "if constexpr (std::is_signed_v<T>)");
        start_block!(
            out,
            "if (len < sizeof(U) * 8 && (result & (U(1) << (len - 1))))"
        );
        line!(out, "result |= ~U(0) << len;");
        end_block!(out, "");
        end_block!(out, "");
        end_block!(out, "return static_cast<T>(result);");
        empty!(out);
    }

    fn extract_be_fn(out: &mut Generator) {
        line!(out, "template <typename T>");
        start_block!(
            out,
            "[[nodiscard]] constexpr T extract_be(const uint8_t* data, std::size_t start, std::size_t end) noexcept"
        );
        line!(out, "using U = std::make_unsigned_t<T>;");
        line!(out, "U result = 0;");
        line!(out, "const std::size_t len = end - start;");
        start_block!(out, "for (std::size_t i = 0; i < len; ++i)");
        line!(out, "const std::size_t bit_idx = start + i;");
        line!(
            out,
            "result = (result << 1) | static_cast<U>((data[bit_idx / 8] >> (7 - bit_idx % 8)) & 0x1u);"
        );
        end_block!(out, "");
        start_block!(out, "if constexpr (std::is_signed_v<T>)");
        start_block!(
            out,
            "if (len < sizeof(U) * 8 && (result & (U(1) << (len - 1))))"
        );
        line!(out, "result |= ~U(0) << len;");
        end_block!(out, "");
        end_block!(out, "");
        end_block!(out, "return static_cast<T>(result);");
        empty!(out);
    }

    fn endian_read_and_write(out: &mut Generator) {
        line!(out, "namespace detail {{");
        empty!(out);

        Self::endian_read_fn(out, "read_le", "big");
        Self::endian_read_fn(out, "read_be", "little");
        Self::endian_write_fn(out, "write_le", "big");
        Self::endian_write_fn(out, "write_be", "little");
        Self::extract_le_fn(out);
        Self::extract_be_fn(out);

        line!(out, "}} // namespace detail");
        empty!(out);
    }

    fn signal_value_enum(out: &mut Generator, signal: &Signal, enum_def: &SignalValueEnum) {
        let name = &signal.name.upper_camel();
        let cpp_type = &signal.physical_type.as_cpp_type();

        start_block!(out, "enum class {} : {}", name, cpp_type);
        for variant in &enum_def.variants {
            line!(out, "{} = {},", variant.description, variant.value)
        }
        end_block!(out, "");
        empty!(out);

        line!(
            out,
            "[[nodiscard]] constexpr std::expected<{}, CanError>",
            name
        );
        start_block!(
            out,
            "{}_from_raw({} v) noexcept",
            name.to_snake_case(),
            cpp_type
        );
        start_block!(out, "switch (v)");
        for variant in &enum_def.variants {
            line!(
                out,
                "case {}: return {}::{};",
                variant.value,
                name,
                variant.description
            );
        }
        end_block!(
            out,
            "default: return std::unexpected(CanError::InvalidData);"
        );
        end_block!(out, "");
        empty!(out);
    }

    fn emit_signal_reads(
        out: &mut Generator,
        signals: &[&Signal],
        file: &DbcFile,
        data_expr: &str,
    ) {
        for signal in signals {
            let layout = &file.signal_layouts[signal.layout.0];
            let raw_type = signal.raw_type.as_cpp_type();
            let phys_type = signal.physical_type.as_cpp_type();
            let field_name = signal.name.0.to_snake_case();
            let raw_name = format!("raw_{}", field_name);
            let extract_fn = match layout.byte_order {
                ByteOrder::LittleEndian => "extract_le",
                ByteOrder::BigEndian => "extract_be",
            };

            line!(
                out,
                "const {} {} = detail::{}<{}>({}, {}, {});",
                raw_type,
                raw_name,
                extract_fn,
                raw_type,
                data_expr,
                layout.bitvec_start,
                layout.bitvec_end
            );

            let is_float = phys_type == "float" || phys_type == "double";

            if signal.signal_value_enum_idx.is_some() {
                let from_fn = format!("{}_from_raw", signal.name.upper_camel().to_snake_case());
                line!(out, "auto {}_exp = {}({});", field_name, from_fn, raw_name);
                line!(
                    out,
                    "if (!{}_exp) return std::unexpected({}_exp.error());",
                    field_name,
                    field_name
                );
            } else if is_float {
                let (factor_str, offset_str) = if phys_type == "float" {
                    (format!("{}f", layout.factor), format!("{}f", layout.offset))
                } else {
                    (format!("{}", layout.factor), format!("{}", layout.offset))
                };
                line!(
                    out,
                    "const {} {} = static_cast<{}>({}) * {} + {};",
                    phys_type,
                    field_name,
                    phys_type,
                    raw_name,
                    factor_str,
                    offset_str
                );
            } else {
                line!(
                    out,
                    "const {} {} = static_cast<{}>({}) * {} + {};",
                    phys_type,
                    field_name,
                    phys_type,
                    raw_name,
                    layout.factor as i64,
                    layout.offset as i64
                );
            }
        }
    }

    fn field_inits_str(signals: &[&Signal]) -> String {
        signals
            .iter()
            .map(|s| {
                let f = s.name.0.to_snake_case();
                if s.signal_value_enum_idx.is_some() {
                    format!(".{} = *{}_exp", f, f)
                } else {
                    format!(".{} = {}", f, f)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn emit_signal_fields(out: &mut Generator, signals: &[&Signal]) {
        for signal in signals {
            if signal.signal_value_enum_idx.is_some() {
                line!(
                    out,
                    "{} {};",
                    signal.name.upper_camel(),
                    signal.name.0.to_snake_case()
                );
            } else {
                line!(
                    out,
                    "{} {};",
                    signal.physical_type.as_cpp_type(),
                    signal.name.lower()
                );
            }
        }
    }

    fn parse_message(out: &mut Generator, msg: &Message, signals: &[&Signal], file: &DbcFile) {
        line!(
            out,
            "[[nodiscard]] static std::expected<{}, CanError>",
            msg.name.upper_camel()
        );
        start_block!(out, "parse(std::span<const uint8_t, LEN> data) noexcept");

        Self::emit_signal_reads(out, signals, file, "data.data()");

        let fields_str = Self::field_inits_str(signals);
        end_block!(
            out,
            "return {}{{ {} }};",
            msg.name.upper_camel(),
            fields_str
        );
        empty!(out);
    }

    fn mux_variant_struct(
        out: &mut Generator,
        msg: &Message,
        mux_value: u64,
        signals: &[&Signal],
        file: &DbcFile,
    ) {
        let struct_name = format!("{}Mux{}", msg.name.upper_camel(), mux_value);

        start_block!(out, "struct {}", struct_name);

        Self::emit_signal_fields(out, signals);
        empty!(out);

        line!(
            out,
            "[[nodiscard]] static std::expected<{}, CanError>",
            struct_name
        );
        start_block!(out, "decode_from(const uint8_t* data) noexcept");
        Self::emit_signal_reads(out, signals, file, "data");
        let fields_str = Self::field_inits_str(signals);
        end_block!(out, "return {}{{ {} }};", struct_name, fields_str);
        empty!(out);

        end_block!(out, "");
        empty!(out);
    }

    fn parse_mux_message(
        out: &mut Generator,
        msg: &Message,
        plain: &[&Signal],
        mux_signal: &Signal,
        muxed: &BTreeMap<u64, Vec<&Signal>>,
        file: &DbcFile,
    ) {
        let msg_name = msg.name.upper_camel();
        let mux_enum_name = format!("{}Mux", msg_name);

        line!(
            out,
            "[[nodiscard]] static std::expected<{}, CanError>",
            msg_name
        );
        start_block!(out, "parse(std::span<const uint8_t, LEN> data) noexcept");

        Self::emit_signal_reads(out, plain, file, "data.data()");

        let mux_layout = &file.signal_layouts[mux_signal.layout.0];
        let mux_raw_type = mux_signal.raw_type.as_cpp_type();
        let mux_extract_fn = match mux_layout.byte_order {
            ByteOrder::LittleEndian => "extract_le",
            ByteOrder::BigEndian => "extract_be",
        };
        line!(
            out,
            "const {} mux_raw = detail::{}<{}>({}, {}, {});",
            mux_raw_type,
            mux_extract_fn,
            mux_raw_type,
            "data.data()",
            mux_layout.bitvec_start,
            mux_layout.bitvec_end
        );

        start_block!(out, "switch (mux_raw)");
        for (mux_value, _) in muxed {
            let variant_struct = format!("{}Mux{}", msg_name, mux_value);
            let variant_name = format!("V{}", mux_value);
            line!(out, "case {}:", mux_value);
            start_block!(out, "");
            line!(
                out,
                "auto inner = {}::decode_from(data.data());",
                variant_struct
            );
            line!(out, "if (!inner) return std::unexpected(inner.error());");
            let plain_fields = Self::field_inits_str(plain);
            let plain_prefix = if plain_fields.is_empty() {
                String::new()
            } else {
                format!("{}, ", plain_fields)
            };
            line!(
                out,
                "return {}{{ {}.mux = *inner }};",
                msg_name,
                plain_prefix,
            );
            end_block!(out, "");
        }
        end_block!(
            out,
            "default: return std::unexpected(CanError::InvalidData);"
        );

        end_block!(out, "");
        empty!(out);
    }

    fn message(out: &mut Generator, msg: &Message, file: &DbcFile) {
        let all_signals: Vec<&Signal> = msg
            .signal_idxs
            .iter()
            .map(|idx| &file.signals[idx.0])
            .collect();

        for signal in &all_signals {
            if let Some(idx) = &signal.signal_value_enum_idx {
                Self::signal_value_enum(out, signal, &file.signal_value_enums[idx.0]);
            }
        }

        match msg.classify_signals(&file.signals) {
            MessageSignalClassification::Plain { signals } => {
                let sigs: Vec<&Signal> = signals.iter().map(|idx| &file.signals[idx.0]).collect();

                start_block!(out, "struct {}", msg.name.upper_camel());
                match msg.id {
                    MessageId::Standard(id) => line!(out, "static constexpr uint16_t ID = {};", id),
                    MessageId::Extended(id) => line!(out, "static constexpr uint32_t ID = {};", id),
                }
                line!(out, "static constexpr std::size_t LEN = {};", msg.size);
                empty!(out);
                Self::emit_signal_fields(out, &sigs);
                empty!(out);
                Self::parse_message(out, msg, &sigs, file);
                end_block!(out, "");
                empty!(out);
            }

            MessageSignalClassification::Multiplexed {
                plain,
                mux_signal: mux_idx,
                muxed,
            } => {
                let plain_sigs: Vec<&Signal> =
                    plain.iter().map(|idx| &file.signals[idx.0]).collect();
                let mux_sig = &file.signals[mux_idx.0];
                let muxed_sigs: BTreeMap<u64, Vec<&Signal>> = muxed
                    .iter()
                    .map(|(v, idxs)| (*v, idxs.iter().map(|idx| &file.signals[idx.0]).collect()))
                    .collect();

                let msg_name = msg.name.upper_camel();
                let mux_enum_name = format!("{}Mux", msg_name);

                for (mux_value, sigs) in &muxed_sigs {
                    Self::mux_variant_struct(out, msg, *mux_value, sigs, file);
                }

                let variant_types = muxed_sigs
                    .keys()
                    .map(|v| format!("{}Mux{}", msg_name, v))
                    .collect::<Vec<_>>()
                    .join(", ");
                line!(
                    out,
                    "using {} = std::variant<{}>;",
                    mux_enum_name,
                    variant_types
                );
                empty!(out);

                start_block!(out, "struct {}", msg_name);
                match msg.id {
                    MessageId::Standard(id) => line!(out, "static constexpr uint16_t ID = {};", id),
                    MessageId::Extended(id) => line!(out, "static constexpr uint32_t ID = {};", id),
                }
                line!(out, "static constexpr std::size_t LEN = {};", msg.size);
                empty!(out);
                Self::emit_signal_fields(out, &plain_sigs);
                line!(out, "{} mux;", mux_enum_name);
                empty!(out);
                Self::parse_mux_message(out, msg, &plain_sigs, mux_sig, &muxed_sigs, file);
                end_block!(out, "");
                empty!(out);
            }
        }
    }
}
