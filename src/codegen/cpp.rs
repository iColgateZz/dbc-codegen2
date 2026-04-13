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
        signal_value_type::{CppType, RawType},
    },
    line, start_block,
};

use crate::codegen::config::CodegenConfig;

pub struct CppGen;

impl CppGen {
    pub fn generate(file: &DbcFile, config: &CodegenConfig) -> String {
        let mut out = Generator::new();

        line!(out, "#pragma once");
        empty!(out);

        Self::includes(&mut out);
        Self::errors(&mut out);
        Self::endian_read_and_write(&mut out);

        for message in &file.messages {
            Self::message(&mut out, message, file, config);
        }

        Self::parse_can(&mut out, &file.messages);

        out.into_string()
    }

    fn cpp_uint_repr_for_float(raw_type: &RawType) -> &'static str {
        match raw_type {
            RawType::Float32 => "uint32_t",
            RawType::Float64 => "uint64_t",
            RawType::Integer(_) => unreachable!("not a float raw type"),
        }
    }

    fn includes(out: &mut Generator) {
        const INCLUDES: &[&str] = &["array", "cstddef", "cstdint", "expected", "span", "variant", "utility"];

        for include in INCLUDES {
            line!(out, "#include <{}>", include);
        }
        empty!(out);
    }

    fn errors(out: &mut Generator) {
        const ERRORS: &[&str] = &[
            "UnknownFrameId",
            "UnknownMuxValue",
            "InvalidPayloadSize",
            "ValueOutOfRange",
            "InvalidEnumValue"
        ];

        start_block!(out, "enum class CanError : uint8_t");
        for error in ERRORS {
            line!(out, "{},", error);
        }
        end_block!(out, "");
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
        line!(out, "if (len == 0) return T(0);");
        start_block!(out, "if (len < sizeof(U) * 8)");
        line!(out, "const U sign_bit = static_cast<U>(U(1) << (len - 1));");
        start_block!(out, "if (result & sign_bit)");
        end_block!(out, "result |= static_cast<U>(~U(0) << len);");
        end_block!(out, "");
        end_block!(out, "");
        end_block!(out, "return static_cast<T>(result);");
        empty!(out);
    }

    fn insert_le_fn(out: &mut Generator) {
        line!(out, "template <typename T>");
        start_block!(
            out,
            "constexpr void insert_le(uint8_t* data, std::size_t start, std::size_t end, T value) noexcept"
        );
        line!(out, "using U = std::make_unsigned_t<T>;");
        line!(out, "U v = static_cast<U>(value);");
        line!(out, "const std::size_t len = end - start;");
        start_block!(out, "for (std::size_t i = 0; i < len; ++i)");
        line!(out, "const std::size_t bit_idx = start + i;");
        line!(
            out,
            "const uint8_t bit = static_cast<uint8_t>((v >> i) & 0x1u);"
        );
        line!(
            out,
            "data[bit_idx / 8] &= ~static_cast<uint8_t>(1u << (bit_idx % 8));"
        );
        line!(
            out,
            "data[bit_idx / 8] |= static_cast<uint8_t>(bit << (bit_idx % 8));"
        );
        end_block!(out, "");
        end_block!(out, "");
        empty!(out);
    }

    fn insert_be_fn(out: &mut Generator) {
        line!(out, "template <typename T>");
        start_block!(
            out,
            "constexpr void insert_be(uint8_t* data, std::size_t start, std::size_t end, T value) noexcept"
        );
        line!(out, "using U = std::make_unsigned_t<T>;");
        line!(out, "U v = static_cast<U>(value);");
        line!(out, "const std::size_t len = end - start;");
        start_block!(out, "for (std::size_t i = 0; i < len; ++i)");
        line!(out, "const std::size_t bit_idx = start + i;");
        line!(
            out,
            "const uint8_t bit = static_cast<uint8_t>((v >> (len - 1 - i)) & 0x1u);"
        );
        line!(
            out,
            "data[bit_idx / 8] &= ~static_cast<uint8_t>(1u << (7 - bit_idx % 8));"
        );
        line!(
            out,
            "data[bit_idx / 8] |= static_cast<uint8_t>(bit << (7 - bit_idx % 8));"
        );
        end_block!(out, "");
        end_block!(out, "");
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
        line!(out, "if (len == 0) return T(0);");
        start_block!(out, "if (len < sizeof(U) * 8)");
        line!(out, "const U sign_bit = static_cast<U>(U(1) << (len - 1));");
        start_block!(out, "if (result & sign_bit)");
        end_block!(out, "result |= static_cast<U>(~U(0) << len);");
        end_block!(out, "");
        end_block!(out, "");
        end_block!(out, "return static_cast<T>(result);");
        empty!(out);
    }

    fn endian_read_and_write(out: &mut Generator) {
        line!(out, "namespace detail {{");
        empty!(out);

        Self::extract_le_fn(out);
        Self::extract_be_fn(out);
        Self::insert_le_fn(out);
        Self::insert_be_fn(out);

        line!(out, "}} // namespace detail");
        empty!(out);
    }

    fn signal_value_enum(out: &mut Generator, signal: &Signal, enum_def: &SignalValueEnum, config: &CodegenConfig) {
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
        if !config.no_enum_other {
            end_block!(
                out,
                "default: return static_cast<{}>(v);",
                name
            );
        } else {
            end_block!(
                out,
                "default: return std::unexpected(CanError::InvalidEnumValue);"
            );
        }
        end_block!(out, "");
        empty!(out);
    }

    fn emit_signal_getters(
        out: &mut Generator,
        signals: &[&Signal],
        file: &DbcFile,
    ) {
        for signal in signals {
            let layout = &file.signal_layouts[signal.layout.0];
            let phys_type = signal.physical_type.as_cpp_type();
            let field_name = signal.name.0.to_snake_case();
            let extract_fn = match layout.byte_order {
                ByteOrder::LittleEndian => "extract_le",
                ByteOrder::BigEndian => "extract_be",
            };
            let is_raw_float = matches!(signal.raw_type, RawType::Float32 | RawType::Float64);
            let is_phys_float = phys_type == "float" || phys_type == "double";

            let return_type = if signal.signal_value_enum_idx.is_some() {
                format!("std::expected<{}, CanError>", signal.name.upper_camel())
            } else {
                phys_type.to_string()
            };

            start_block!(out, "[[nodiscard]] {} {}() const noexcept", return_type, field_name);
            let data_expr = "data_.data()";

            if signal.signal_value_enum_idx.is_some() {
                let raw_type = signal.raw_type.as_cpp_type();
                let raw_name = format!("raw_{}", field_name);
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
                let from_fn = format!("{}_from_raw", signal.name.upper_camel().to_snake_case());
                line!(out, "return {}({});", from_fn, raw_name);
            } else if is_raw_float {
                let uint_repr = Self::cpp_uint_repr_for_float(&signal.raw_type);
                let (factor_str, offset_str) = if phys_type == "float" {
                    (format!("{}f", layout.factor), format!("{}f", layout.offset))
                } else {
                    (format!("{}", layout.factor), format!("{}", layout.offset))
                };
                line!(
                    out,
                    "const {} {}_bits = detail::{}<{}>({}, {}, {});",
                    uint_repr,
                    field_name,
                    extract_fn,
                    uint_repr,
                    data_expr,
                    layout.bitvec_start,
                    layout.bitvec_end
                );
                line!(
                    out,
                    "{} {}_raw; std::memcpy(&{}_raw, &{}_bits, sizeof({}));",
                    phys_type,
                    field_name,
                    field_name,
                    field_name,
                    phys_type
                );
                line!(
                    out,
                    "return {}_raw * {} + {};",
                    field_name,
                    factor_str,
                    offset_str
                );
            } else if is_phys_float {
                let raw_type = signal.raw_type.as_cpp_type();
                let raw_name = format!("raw_{}", field_name);
                let (factor_str, offset_str) = if phys_type == "float" {
                    (format!("{}f", layout.factor), format!("{}f", layout.offset))
                } else {
                    (format!("{}", layout.factor), format!("{}", layout.offset))
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
                line!(
                    out,
                    "return static_cast<{}>({}) * {} + {};",
                    phys_type,
                    raw_name,
                    factor_str,
                    offset_str
                );
            } else {
                let raw_type = signal.raw_type.as_cpp_type();
                let raw_name = format!("raw_{}", field_name);
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
                line!(
                    out,
                    "return static_cast<{}>({}) * {} + {};",
                    phys_type,
                    raw_name,
                    layout.factor as i64,
                    layout.offset as i64
                );
            }
            end_block!(out, "");
            empty!(out);
        }
    }

    fn emit_setter_range_check(out: &mut Generator, signal: &Signal, file: &DbcFile, config: &CodegenConfig, field_name: &str) {
        if signal.signal_value_enum_idx.is_some() {
            return;
        }

        let layout = &file.signal_layouts[signal.layout.0];
        let min = layout.min;
        let max = layout.max;

        if config.zero_zero_range_allows_all && min == max && min == 0.0 {
            return;
        }

        let phys_type = signal.physical_type.as_cpp_type();
        let is_phys_float = phys_type == "float" || phys_type == "double";
        
        let min_str = if is_phys_float { format!("{min}f") } else { format!("{}", min as i64) };
        let max_str = if is_phys_float { format!("{max}f") } else { format!("{}", max as i64) };

        line!(out, "if ({} < {} || {} > {}) return std::unexpected(CanError::ValueOutOfRange);", field_name, min_str, field_name, max_str);
    }

    fn emit_signal_setters(
        out: &mut Generator,
        signals: &[&Signal],
        file: &DbcFile,
        config: &CodegenConfig,
    ) {
        for signal in signals {
            let layout = &file.signal_layouts[signal.layout.0];
            let phys_type = signal.physical_type.as_cpp_type();
            let field_name = signal.name.0.to_snake_case();
            let insert_fn = match layout.byte_order {
                ByteOrder::LittleEndian => "insert_le",
                ByteOrder::BigEndian => "insert_be",
            };
            let is_raw_float = matches!(signal.raw_type, RawType::Float32 | RawType::Float64);
            let is_phys_float = phys_type == "float" || phys_type == "double";

            let param_type = if signal.signal_value_enum_idx.is_some() {
                format!("{}", signal.name.upper_camel())
            } else {
                phys_type.to_string()
            };

            start_block!(out, "std::expected<void, CanError> set_{}({} {}) noexcept", field_name, param_type, field_name);
            Self::emit_setter_range_check(out, signal, file, config, &field_name);
            
            let data_expr = "data_.data()";

            if signal.signal_value_enum_idx.is_some() {
                let raw_type = signal.raw_type.as_cpp_type();
                line!(
                    out,
                    "detail::{}<{}>({}, {}, {}, static_cast<{}>({}));",
                    insert_fn,
                    raw_type,
                    data_expr,
                    layout.bitvec_start,
                    layout.bitvec_end,
                    raw_type,
                    field_name
                );
            } else if is_raw_float {
                let uint_repr = Self::cpp_uint_repr_for_float(&signal.raw_type);
                let (factor_str, offset_str) = if phys_type == "float" {
                    (format!("{}f", layout.factor), format!("{}f", layout.offset))
                } else {
                    (format!("{}", layout.factor), format!("{}", layout.offset))
                };
                line!(
                    out,
                    "const {} {}_raw = ({} - {}) / {};",
                    phys_type,
                    field_name,
                    field_name,
                    offset_str,
                    factor_str
                );
                line!(
                    out,
                    "{} {}_bits; std::memcpy(&{}_bits, &{}_raw, sizeof({}));",
                    uint_repr,
                    field_name,
                    field_name,
                    field_name,
                    uint_repr
                );
                line!(
                    out,
                    "detail::{}<{}>({}, {}, {}, {}_bits);",
                    insert_fn,
                    uint_repr,
                    data_expr,
                    layout.bitvec_start,
                    layout.bitvec_end,
                    field_name
                );
            } else if is_phys_float {
                let raw_type = signal.raw_type.as_cpp_type();
                let (factor_str, offset_str) = if phys_type == "float" {
                    (format!("{}f", layout.factor), format!("{}f", layout.offset))
                } else {
                    (format!("{}", layout.factor), format!("{}", layout.offset))
                };
                line!(
                    out,
                    "detail::{}<{}>({}, {}, {}, static_cast<{}>(({} - {}) / {}));",
                    insert_fn,
                    raw_type,
                    data_expr,
                    layout.bitvec_start,
                    layout.bitvec_end,
                    raw_type,
                    field_name,
                    offset_str,
                    factor_str
                );
            } else {
                let raw_type = signal.raw_type.as_cpp_type();
                line!(
                    out,
                    "detail::{}<{}>({}, {}, {}, static_cast<{}>(({} - {}) / {}));",
                    insert_fn,
                    raw_type,
                    data_expr,
                    layout.bitvec_start,
                    layout.bitvec_end,
                    raw_type,
                    field_name,
                    layout.offset as i64,
                    layout.factor as i64
                );
            }
            end_block!(out, "return {{}};");
            empty!(out);
        }
    }

    fn emit_create_method(out: &mut Generator, msg_name: &str, signals: &[&Signal]) {
        let args = signals
            .iter()
            .map(|s| {
                if s.signal_value_enum_idx.is_some() {
                    format!("{} {}", s.name.upper_camel(), s.name.0.to_snake_case())
                } else {
                    format!("{} {}", s.physical_type.as_cpp_type(), s.name.0.to_snake_case())
                }
            })
            .collect::<Vec<_>>()
            .join(",
            ");
        
        let args_formatted = if args.is_empty() { String::new() } else { format!("
            {}
        ", args) };

        start_block!(out, "[[nodiscard]] static std::expected<{}, CanError> create({}) noexcept", msg_name, args_formatted);
        line!(out, "{} msg{{}};", msg_name);
        for signal in signals {
            let f = signal.name.0.to_snake_case();
            line!(out, "if (auto r = msg.set_{}({}); !r) return std::unexpected(r.error());", f, f);
        }
        end_block!(out, "return msg;");
        empty!(out);
    }

    fn mux_variant_class(
        out: &mut Generator,
        msg: &Message,
        mux_value: u64,
        signals: &[&Signal],
        file: &DbcFile,
        config: &CodegenConfig,
    ) {
        let class_name = format!("{}Mux{}", msg.name.upper_camel(), mux_value);

        start_block!(out, "class {}", class_name);

        line!(out, "public:");

        line!(out, "static constexpr std::size_t LEN = {};", msg.size);
        empty!(out);

        Self::emit_create_method(out, &class_name, signals);

        Self::emit_signal_getters(out, signals, file);
        Self::emit_signal_setters(out, signals, file, config);

        line!(out, "[[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept {{ return data_; }}");
        empty!(out);

        // Required for multiplex setter
        line!(out, "private:");
        line!(out, "friend class {};", msg.name.upper_camel());
        line!(out, "std::array<uint8_t, LEN> data_{{}};");

        end_block!(out, "");
        empty!(out);
    }

    fn message(out: &mut Generator, msg: &Message, file: &DbcFile, config: &CodegenConfig) {
        let all_signals: Vec<&Signal> = msg
            .signal_idxs
            .iter()
            .map(|idx| &file.signals[idx.0])
            .collect();

        for signal in &all_signals {
            if let Some(idx) = &signal.signal_value_enum_idx {
                Self::signal_value_enum(out, signal, &file.signal_value_enums[idx.0], config);
            }
        }

        match msg.classify_signals(&file.signals) {
            MessageSignalClassification::Plain { signals } => {
                let sigs: Vec<&Signal> = signals.iter().map(|idx| &file.signals[idx.0]).collect();
                let msg_name = msg.name.upper_camel();

                start_block!(out, "class {}", msg_name);
                line!(out, "public:");
                
                match msg.id {
                    MessageId::Standard(id) => line!(out, "static constexpr uint16_t ID = {};", id),
                    MessageId::Extended(id) => line!(out, "static constexpr uint32_t ID = {};", id),
                }
                line!(out, "static constexpr std::size_t LEN = {};", msg.size);
                empty!(out);
                
                Self::emit_create_method(out, &msg_name, &sigs);

                start_block!(out, "[[nodiscard]] static std::expected<{}, CanError> try_from_frame(std::span<const uint8_t> frame) noexcept", msg_name);
                line!(out, "if (frame.size() < LEN) return std::unexpected(CanError::InvalidPayloadSize);");
                line!(out, "{} msg{{}};", msg_name);
                line!(out, "std::memcpy(msg.data_.data(), frame.data(), LEN);");
                end_block!(out, "return msg;");
                empty!(out);
                
                Self::emit_signal_getters(out, &sigs, file);
                Self::emit_signal_setters(out, &sigs, file, config);

                line!(out, "[[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept {{ return data_; }}");
                empty!(out);

                line!(out, "private:");
                line!(out, "std::array<uint8_t, LEN> data_{{}};");

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
                    Self::mux_variant_class(out, msg, *mux_value, sigs, file, config);
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

                start_block!(out, "class {}", msg_name);
                line!(out, "public:");
                
                match msg.id {
                    MessageId::Standard(id) => line!(out, "static constexpr uint16_t ID = {};", id),
                    MessageId::Extended(id) => line!(out, "static constexpr uint32_t ID = {};", id),
                }
                line!(out, "static constexpr std::size_t LEN = {};", msg.size);
                empty!(out);
                
                start_block!(out, "[[nodiscard]] static std::expected<{}, CanError> try_from_frame(std::span<const uint8_t> frame) noexcept", msg_name);
                line!(out, "if (frame.size() < LEN) return std::unexpected(CanError::InvalidPayloadSize);");
                line!(out, "{} msg{{}};", msg_name);
                line!(out, "std::memcpy(msg.data_.data(), frame.data(), LEN);");
                end_block!(out, "return msg;");
                empty!(out);
                
                Self::emit_signal_getters(out, &plain_sigs, file);
                Self::emit_signal_setters(out, &plain_sigs, file, config);

                let mux_layout = &file.signal_layouts[mux_sig.layout.0];
                let mux_raw_type = mux_sig.raw_type.as_cpp_type();
                let mux_extract_fn = match mux_layout.byte_order {
                    ByteOrder::LittleEndian => "extract_le",
                    ByteOrder::BigEndian => "extract_be",
                };
                let mux_insert_fn = match mux_layout.byte_order {
                    ByteOrder::LittleEndian => "insert_le",
                    ByteOrder::BigEndian => "insert_be",
                };
                
                // Mux Getter
                start_block!(out, "[[nodiscard]] std::expected<{}, CanError> mux() const noexcept", mux_enum_name);
                line!(out, "const {} mux_raw = detail::{}<{}>({}, {}, {});", 
                    mux_raw_type, mux_extract_fn, mux_raw_type, "data_.data()", mux_layout.bitvec_start, mux_layout.bitvec_end);
                start_block!(out, "switch (mux_raw)");
                for (mux_value, _) in &muxed_sigs {
                    let variant_class = format!("{}Mux{}", msg_name, mux_value);
                    start_block!(out, "case {}:", mux_value);
                    line!(out, "{} inner{{}};", variant_class);
                    line!(out, "std::memcpy(inner.data_.data(), data_.data(), LEN);");
                    end_block!(out, "return inner;");
                }
                end_block!(out, "default: return std::unexpected(CanError::UnknownMuxValue);");
                end_block!(out, "");
                empty!(out);
                
                // Mux Setters
                for (mux_value, _) in &muxed_sigs {
                    let variant_class = format!("{}Mux{}", msg_name, mux_value);
                    start_block!(out, "void set_mux_{}(const {}& value) noexcept", mux_value, variant_class);
                    line!(out, "for (std::size_t i = 0; i < LEN; ++i) data_[i] |= value.data_[i];");
                    line!(
                        out,
                        "detail::{}<{}>({}, {}, {}, static_cast<{}>({}));",
                        mux_insert_fn,
                        mux_raw_type,
                        "data_.data()",
                        mux_layout.bitvec_start,
                        mux_layout.bitvec_end,
                        mux_raw_type,
                        mux_value
                    );
                    end_block!(out, "");
                }
                empty!(out);

                line!(out, "[[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept {{ return data_; }}");
                empty!(out);

                line!(out, "private:");
                line!(out, "std::array<uint8_t, LEN> data_{{}};");

                end_block!(out, ";");
                empty!(out);
            }
        }
    }

    fn parse_can(out: &mut Generator, messages: &[Message]) {
        let variant_types = messages
            .iter()
            .map(|m| m.name.upper_camel())
            .collect::<Vec<_>>()
            .join(", ");
        line!(out, "using CanMsg = std::variant<{}>;", variant_types);
        empty!(out);

        line!(out, "[[nodiscard]]");
        line!(out, "inline std::expected<CanMsg, CanError>");
        start_block!(
            out,
            "parse_can(uint32_t id, std::span<const uint8_t> frame) noexcept"
        );

        start_block!(out, "switch (id)");

        for msg in messages {
            let name = msg.name.upper_camel();
            line!(out, "case {}::ID:", name);
            start_block!(out, "");
            line!(
                out,
                "auto r = {}::try_from_frame(frame);",
                name
            );
            line!(out, "if (!r) return std::unexpected(r.error());");
            line!(out, "return CanMsg{{std::move(*r)}};");
            end_block!(out, "");
        }

        end_block!(out, "default: return std::unexpected(CanError::UnknownFrameId);");
        end_block!(out, "");
    }
}
