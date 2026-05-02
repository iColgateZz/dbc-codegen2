use std::collections::BTreeMap;

use heck::ToSnakeCase;

use crate::{
    DbcFile,
    codegen::Generator,
    empty, end_block, end_block_no_close,
    ir::{
        message::{Message, MessageId, MessageSignalClassification},
        signal::Signal,
        signal_layout::{ByteOrder, SignalLayout},
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
        Self::can_id(&mut out);
        Self::message_interface(&mut out);
        Self::endian_read_and_write(&mut out);

        let mut emitted_enum_idxs = std::collections::BTreeSet::new();
        for signal in &file.signals {
            if let Some(idx) = signal.signal_value_enum_idx {
                if emitted_enum_idxs.insert(idx.0) {
                    Self::signal_value_enum(
                        &mut out,
                        signal,
                        &file.signal_value_enums[idx.0],
                        config,
                    );
                }
            }
        }

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

    fn format_cpp_float(val: f64, phys_type: &str) -> String {
        let mut s = format!("{}", val);
        if !s.contains('.') && !s.contains('e') && !s.contains('E') {
            s.push_str(".0");
        }
        if phys_type == "float" {
            s.push('f');
        }
        s
    }

    fn detail_extract_fn(byte_order: ByteOrder) -> &'static str {
        match byte_order {
            ByteOrder::LittleEndian => "extract_le",
            ByteOrder::BigEndian => "extract_be",
        }
    }

    fn detail_insert_fn(byte_order: ByteOrder) -> &'static str {
        match byte_order {
            ByteOrder::LittleEndian => "insert_le",
            ByteOrder::BigEndian => "insert_be",
        }
    }

    fn detail_copy_fn(byte_order: ByteOrder) -> &'static str {
        match byte_order {
            ByteOrder::LittleEndian => "copy_le",
            ByteOrder::BigEndian => "copy_be",
        }
    }

    fn is_phys_float(phys_type: &str) -> bool {
        phys_type == "float" || phys_type == "double"
    }

    fn is_raw_float(raw_type: &RawType) -> bool {
        matches!(raw_type, RawType::Float32 | RawType::Float64)
    }

    fn is_bool_signal(signal: &Signal, file: &DbcFile) -> bool {
        let layout = &file.signal_layouts[signal.layout.0];
        layout.size == 1 && signal.signal_value_enum_idx.is_none()
    }

    fn signal_cpp_param_type(signal: &Signal, file: &DbcFile) -> String {
        if let Some(idx) = signal.signal_value_enum_idx {
            file.signal_value_enums[idx.0].name.upper_camel()
        } else if Self::is_bool_signal(signal, file) {
            "bool".to_string()
        } else {
            signal.physical_type.as_cpp_type().to_string()
        }
    }

    fn signal_cpp_return_type(signal: &Signal, file: &DbcFile, config: &CodegenConfig) -> String {
        if let Some(idx) = signal.signal_value_enum_idx {
            let enum_name = file.signal_value_enums[idx.0].name.upper_camel();
            if config.no_enum_other {
                format!("std::expected<{}, CanError>", enum_name)
            } else {
                enum_name
            }
        } else if Self::is_bool_signal(signal, file) {
            "bool".to_string()
        } else {
            signal.physical_type.as_cpp_type().to_string()
        }
    }

    fn signal_param_decl(signal: &Signal, file: &DbcFile) -> String {
        format!(
            "{} {}",
            Self::signal_cpp_param_type(signal, file),
            signal.name.raw.to_snake_case()
        )
    }

    fn format_function_args(args: &[String], multiline: bool) -> String {
        if args.is_empty() {
            return String::new();
        }

        if multiline {
            format!("\n            {}\n        ", args.join(",\n            "))
        } else {
            args.join(", ")
        }
    }

    fn emit_detail_extract(
        out: &mut Generator,
        cpp_type: &str,
        var_name: &str,
        detail_fn: &str,
        data_expr: &str,
        layout: &SignalLayout,
    ) {
        line!(
            out,
            "const {} {} = detail::{}<{}>({}, {}, {});",
            cpp_type,
            var_name,
            detail_fn,
            cpp_type,
            data_expr,
            layout.bitvec_start,
            layout.bitvec_end
        );
    }

    fn emit_detail_insert(
        out: &mut Generator,
        cpp_type: &str,
        detail_fn: &str,
        data_expr: &str,
        layout: &SignalLayout,
        value_expr: &str,
    ) {
        line!(
            out,
            "detail::{}<{}>({}, {}, {}, {});",
            detail_fn,
            cpp_type,
            data_expr,
            layout.bitvec_start,
            layout.bitvec_end,
            value_expr
        );
    }

    fn emit_detail_copy(
        out: &mut Generator,
        detail_fn: &str,
        dst_expr: &str,
        src_expr: &str,
        layout: &SignalLayout,
    ) {
        line!(
            out,
            "detail::{}({}, {}, {}, {});",
            detail_fn,
            dst_expr,
            src_expr,
            layout.bitvec_start,
            layout.bitvec_end
        );
    }

    fn emit_message_id(out: &mut Generator, msg: &Message) {
        match msg.id {
            MessageId::Standard(id) => {
                line!(out, "static constexpr CanId ID = CanId::standard({});", id)
            }
            MessageId::Extended(id) => {
                line!(out, "static constexpr CanId ID = CanId::extended({});", id)
            }
        }
    }

    fn emit_len_constant(out: &mut Generator, len: u64) {
        line!(out, "static constexpr std::size_t LEN{{{}}};", len);
    }

    fn emit_try_from_frame(out: &mut Generator, msg_name: &str) {
        start_block!(
            out,
            "[[nodiscard]] static std::expected<{}, CanError> try_from_frame(CanId id, std::span<const uint8_t> frame) noexcept",
            msg_name
        );
        line!(
            out,
            "if (frame.size() < LEN) return std::unexpected(CanError::InvalidPayloadSize);"
        );
        line!(
            out,
            "if (id != ID) return std::unexpected(CanError::InvalidFrameId);"
        );
        line!(out, "{} msg{{}};", msg_name);
        line!(out, "std::memcpy(msg.data_.data(), frame.data(), LEN);");
        end_block!(out, "return msg;");
        empty!(out);
    }

    fn emit_encode_method(out: &mut Generator) {
        line!(
            out,
            "[[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept {{ return data_; }}"
        );
        empty!(out);
    }

    fn emit_data_storage(out: &mut Generator, friend_class: Option<&str>) {
        line!(out, "private:");
        if let Some(friend_class) = friend_class {
            line!(out, "friend class {};", friend_class);
        }
        line!(out, "std::array<uint8_t, LEN> data_{{}};");
    }

    fn emit_signal_set_calls(out: &mut Generator, target: &str, signals: &[&Signal]) {
        for signal in signals {
            let field_name = signal.name.raw.to_snake_case();
            line!(
                out,
                "if (auto r = {}.set_{}({}); !r) return std::unexpected(r.error());",
                target,
                field_name,
                field_name
            );
        }
    }

    fn includes(out: &mut Generator) {
        const INCLUDES: &[&str] = &[
            "array", "cstddef", "cstdint", "expected", "span", "variant", "utility", "cstring",
        ];

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
            "InvalidFrameId",
            "ValueOutOfRange",
            "InvalidEnumValue",
        ];

        start_block!(out, "enum class CanError : uint8_t");
        for error in ERRORS {
            line!(out, "{},", error);
        }
        end_block!(out, "");
        empty!(out);
    }

    fn can_id(out: &mut Generator) {
        start_block!(out, "struct CanId");
        start_block!(out, "enum class Kind : uint8_t");
        line!(out, "Standard,");
        line!(out, "Extended,");
        end_block!(out, "");
        empty!(out);

        line!(out, "uint32_t value;");
        line!(out, "Kind kind;");
        empty!(out);

        line!(
            out,
            "[[nodiscard]] static constexpr CanId standard(uint32_t value) noexcept {{ return CanId{{value, Kind::Standard}}; }}"
        );
        line!(
            out,
            "[[nodiscard]] static constexpr CanId extended(uint32_t value) noexcept {{ return CanId{{value, Kind::Extended}}; }}"
        );
        line!(
            out,
            "[[nodiscard]] static constexpr CanId from_raw(uint32_t value, bool is_extended) noexcept {{ return is_extended ? extended(value) : standard(value); }}"
        );
        empty!(out);

        line!(
            out,
            "[[nodiscard]] constexpr uint32_t dispatch_key() const noexcept {{ return value | (kind == Kind::Extended ? 0x80000000u : 0u); }}"
        );
        line!(
            out,
            "constexpr bool operator==(const CanId&) const noexcept = default;"
        );
        end_block!(out, "");
        empty!(out);
    }

    fn message_interface(out: &mut Generator) {
        line!(out, "template <typename Msg>");
        line!(
            out,
            "concept GeneratedCanMessage = requires(CanId id, std::span<const uint8_t> frame, const Msg& msg) {{"
        );
        line!(out, "  Msg::ID;");
        line!(out, "  Msg::LEN;");
        line!(out, "  {{ Msg::try_from_frame(id, frame) }};");
        line!(out, "  {{ msg.encode() }};");
        line!(out, "}};");
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

    fn copy_le_fn(out: &mut Generator) {
        start_block!(
            out,
            "constexpr void copy_le(uint8_t* dst, const uint8_t* src, std::size_t start, std::size_t end) noexcept"
        );
        start_block!(
            out,
            "for (std::size_t bit_idx = start; bit_idx < end; ++bit_idx)"
        );
        line!(out, "const std::size_t byte_idx = bit_idx / 8;");
        line!(
            out,
            "const uint8_t mask = static_cast<uint8_t>(1u << (bit_idx % 8));"
        );
        line!(out, "dst[byte_idx] &= static_cast<uint8_t>(~mask);");
        line!(
            out,
            "dst[byte_idx] |= static_cast<uint8_t>(src[byte_idx] & mask);"
        );
        end_block!(out, "");
        end_block!(out, "");
        empty!(out);
    }

    fn copy_be_fn(out: &mut Generator) {
        start_block!(
            out,
            "constexpr void copy_be(uint8_t* dst, const uint8_t* src, std::size_t start, std::size_t end) noexcept"
        );
        start_block!(
            out,
            "for (std::size_t bit_idx = start; bit_idx < end; ++bit_idx)"
        );
        line!(out, "const std::size_t byte_idx = bit_idx / 8;");
        line!(
            out,
            "const uint8_t mask = static_cast<uint8_t>(1u << (7 - bit_idx % 8));"
        );
        line!(out, "dst[byte_idx] &= static_cast<uint8_t>(~mask);");
        line!(
            out,
            "dst[byte_idx] |= static_cast<uint8_t>(src[byte_idx] & mask);"
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
        Self::copy_le_fn(out);
        Self::copy_be_fn(out);

        line!(out, "}} // namespace detail");
        empty!(out);
    }

    fn signal_value_enum(
        out: &mut Generator,
        signal: &Signal,
        enum_def: &SignalValueEnum,
        config: &CodegenConfig,
    ) {
        let name = &enum_def.name.upper_camel();
        let cpp_type = &signal.physical_type.as_cpp_type();

        start_block!(out, "enum class {} : {}", name, cpp_type);
        for variant in &enum_def.variants {
            line!(out, "{} = {},", variant.description, variant.value);
        }
        end_block!(out, "");
        empty!(out);

        if config.no_enum_other {
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
                "default: return std::unexpected(CanError::InvalidEnumValue);"
            );
            end_block!(out, "");
        } else {
            line!(out, "[[nodiscard]] constexpr {}", name);
            start_block!(
                out,
                "{}_from_raw({} v) noexcept",
                name.to_snake_case(),
                cpp_type
            );
            line!(out, "return static_cast<{}>(v);", name);
            end_block!(out, "");
        }
        empty!(out);
    }

    fn emit_message_doc(out: &mut Generator, msg: &Message) {
        let name = &msg.name.raw;

        let id_text = match msg.id {
            MessageId::Standard(id) => format!("Standard {} (0x{:X})", id, id),
            MessageId::Extended(id) => format!("Extended {} (0x{:X})", id, id),
        };

        let size = msg.size;
        let transmitter = match &msg.transmitter {
            crate::ir::message::Transmitter::Node(name) => name.raw.clone(),
            crate::ir::message::Transmitter::VectorXXX => "VectorXXX".to_string(),
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

        line!(out, "/**");
        for l in lines {
            line!(out, " * {}", l);
        }
        line!(out, " */");
    }

    fn emit_signal_doc(out: &mut Generator, signal: &Signal, file: &DbcFile) {
        let layout = &file.signal_layouts[signal.layout.0];

        let name = &signal.name.raw;
        let min = layout.min;
        let max = layout.max;
        let unit = &signal.unit;
        let receivers = if signal.receivers.is_empty() {
            "".into()
        } else {
            signal
                .receivers
                .iter()
                .map(|r| match r {
                    crate::ir::signal::Receiver::Node(id) => id.raw.clone(),
                    crate::ir::signal::Receiver::VectorXXX => "VectorXXX".to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        };

        let start = layout.bitvec_start;
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
            format!("{}", name),
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

        if let Some(comment) = &signal.comment {
            lines.push("".into());
            lines.extend(comment.lines().map(|l| l.to_string()));
        }

        line!(out, "/**");
        for l in lines {
            line!(out, " * {}", l);
        }
        line!(out, " */");
    }

    fn emit_signal_getters(
        out: &mut Generator,
        signals: &[&Signal],
        file: &DbcFile,
        config: &CodegenConfig,
    ) {
        for signal in signals {
            Self::emit_signal_doc(out, signal, file);
            let layout = &file.signal_layouts[signal.layout.0];
            let phys_type = signal.physical_type.as_cpp_type();
            let field_name = signal.name.raw.to_snake_case();
            let extract_fn = Self::detail_extract_fn(layout.byte_order);
            let is_raw_float = Self::is_raw_float(&signal.raw_type);
            let is_phys_float = Self::is_phys_float(phys_type);
            let is_bool_signal = Self::is_bool_signal(signal, file);
            let return_type = Self::signal_cpp_return_type(signal, file, config);

            start_block!(
                out,
                "[[nodiscard]] {} {}() const noexcept",
                return_type,
                field_name
            );
            let data_expr = "data_.data()";

            if is_bool_signal {
                let raw_name = format!("raw_{}", field_name);
                Self::emit_detail_extract(out, "uint8_t", &raw_name, extract_fn, data_expr, layout);
                line!(out, "return {} != 0;", raw_name);
            } else if signal.signal_value_enum_idx.is_some() {
                let raw_type = signal.raw_type.as_cpp_type();
                let raw_name = format!("raw_{}", field_name);
                Self::emit_detail_extract(out, raw_type, &raw_name, extract_fn, data_expr, layout);
                let from_fn = {
                    let idx = signal.signal_value_enum_idx.unwrap();
                    let enum_name = file.signal_value_enums[idx.0].name.upper_camel();
                    format!("{}_from_raw", enum_name.to_snake_case())
                };
                line!(out, "return {}({});", from_fn, raw_name);
            } else if is_raw_float {
                let uint_repr = Self::cpp_uint_repr_for_float(&signal.raw_type);
                let factor_str = Self::format_cpp_float(layout.factor, phys_type);
                let offset_str = Self::format_cpp_float(layout.offset, phys_type);
                Self::emit_detail_extract(
                    out,
                    uint_repr,
                    &format!("{}_bits", field_name),
                    extract_fn,
                    data_expr,
                    layout,
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
                let factor_str = Self::format_cpp_float(layout.factor, phys_type);
                let offset_str = Self::format_cpp_float(layout.offset, phys_type);
                Self::emit_detail_extract(out, raw_type, &raw_name, extract_fn, data_expr, layout);
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
                Self::emit_detail_extract(out, raw_type, &raw_name, extract_fn, data_expr, layout);
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

    fn emit_setter_range_check(
        out: &mut Generator,
        signal: &Signal,
        file: &DbcFile,
        config: &CodegenConfig,
        field_name: &str,
    ) {
        if signal.signal_value_enum_idx.is_some() {
            return;
        }

        let layout = &file.signal_layouts[signal.layout.0];
        if layout.size == 1 {
            return;
        }
        let min = layout.min;
        let max = layout.max;

        if config.zero_zero_range_allows_all && min == max && min == 0.0 {
            return;
        }

        let phys_type = signal.physical_type.as_cpp_type();
        let is_phys_float = phys_type == "float" || phys_type == "double";

        let min_str = if is_phys_float {
            Self::format_cpp_float(min, phys_type)
        } else {
            format!("{}", min as i64)
        };
        let max_str = if is_phys_float {
            Self::format_cpp_float(max, phys_type)
        } else {
            format!("{}", max as i64)
        };

        line!(
            out,
            "if ({} < {} || {} > {}) return std::unexpected(CanError::ValueOutOfRange);",
            field_name,
            min_str,
            field_name,
            max_str
        );
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
            let field_name = signal.name.raw.to_snake_case();
            let insert_fn = Self::detail_insert_fn(layout.byte_order);
            let is_raw_float = Self::is_raw_float(&signal.raw_type);
            let is_phys_float = Self::is_phys_float(phys_type);
            let is_bool_signal = Self::is_bool_signal(signal, file);
            let param_type = Self::signal_cpp_param_type(signal, file);

            start_block!(
                out,
                "std::expected<void, CanError> set_{}({} {}) noexcept",
                field_name,
                param_type,
                field_name
            );
            Self::emit_setter_range_check(out, signal, file, config, &field_name);

            let data_expr = "data_.data()";

            if is_bool_signal {
                Self::emit_detail_insert(
                    out,
                    "uint8_t",
                    insert_fn,
                    data_expr,
                    layout,
                    &format!("static_cast<uint8_t>({})", field_name),
                );
            } else if signal.signal_value_enum_idx.is_some() {
                let raw_type = signal.raw_type.as_cpp_type();
                let encode_expr = format!("static_cast<{}>({})", raw_type, field_name);
                Self::emit_detail_insert(out, raw_type, insert_fn, data_expr, layout, &encode_expr);
            } else if is_raw_float {
                let uint_repr = Self::cpp_uint_repr_for_float(&signal.raw_type);
                let factor_str = Self::format_cpp_float(layout.factor, phys_type);
                let offset_str = Self::format_cpp_float(layout.offset, phys_type);
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
                Self::emit_detail_insert(
                    out,
                    uint_repr,
                    insert_fn,
                    data_expr,
                    layout,
                    &format!("{}_bits", field_name),
                );
            } else if is_phys_float {
                let raw_type = signal.raw_type.as_cpp_type();
                let factor_str = Self::format_cpp_float(layout.factor, phys_type);
                let offset_str = Self::format_cpp_float(layout.offset, phys_type);
                Self::emit_detail_insert(
                    out,
                    raw_type,
                    insert_fn,
                    data_expr,
                    layout,
                    &format!(
                        "static_cast<{}>(({} - {}) / {})",
                        raw_type, field_name, offset_str, factor_str
                    ),
                );
            } else {
                let raw_type = signal.raw_type.as_cpp_type();
                Self::emit_detail_insert(
                    out,
                    raw_type,
                    insert_fn,
                    data_expr,
                    layout,
                    &format!(
                        "static_cast<{}>(({} - {}) / {})",
                        raw_type, field_name, layout.offset as i64, layout.factor as i64
                    ),
                );
            }
            end_block!(out, "return {{}};");
            empty!(out);
        }
    }

    fn emit_create_method(
        out: &mut Generator,
        msg_name: &str,
        signals: &[&Signal],
        file: &DbcFile,
    ) {
        let args = signals
            .iter()
            .map(|signal| Self::signal_param_decl(signal, file))
            .collect::<Vec<_>>();
        let args_formatted = Self::format_function_args(&args, true);

        start_block!(
            out,
            "[[nodiscard]] static std::expected<{}, CanError> create({}) noexcept",
            msg_name,
            args_formatted
        );
        line!(out, "{} msg{{}};", msg_name);
        Self::emit_signal_set_calls(out, "msg", signals);
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

        Self::emit_len_constant(out, msg.size);
        empty!(out);

        Self::emit_create_method(out, &class_name, signals, file);

        Self::emit_signal_getters(out, signals, file, config);
        Self::emit_signal_setters(out, signals, file, config);

        Self::emit_encode_method(out);

        // Required for multiplex setter
        Self::emit_data_storage(out, Some(&msg.name.upper_camel()));

        end_block!(out, "");
        empty!(out);
    }

    fn message(out: &mut Generator, msg: &Message, file: &DbcFile, config: &CodegenConfig) {
        match msg.classify_signals(&file.signals) {
            MessageSignalClassification::Plain { signals } => {
                let sigs: Vec<&Signal> = signals.iter().map(|idx| &file.signals[idx.0]).collect();
                let msg_name = msg.name.upper_camel();

                Self::emit_message_doc(out, msg);
                start_block!(out, "class {}", msg_name);
                line!(out, "public:");

                Self::emit_message_id(out, msg);
                Self::emit_len_constant(out, msg.size);
                empty!(out);

                Self::emit_create_method(out, &msg_name, &sigs, file);

                Self::emit_try_from_frame(out, &msg_name);

                Self::emit_signal_getters(out, &sigs, file, config);
                Self::emit_signal_setters(out, &sigs, file, config);

                Self::emit_encode_method(out);

                Self::emit_data_storage(out, None);

                end_block!(out, "");
                line!(out, "static_assert(GeneratedCanMessage<{}>);", msg_name);
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

                Self::emit_message_doc(out, msg);
                start_block!(out, "class {}", msg_name);
                line!(out, "public:");

                Self::emit_message_id(out, msg);
                Self::emit_len_constant(out, msg.size);
                empty!(out);

                let mut args = plain_sigs
                    .iter()
                    .map(|signal| Self::signal_param_decl(signal, file))
                    .collect::<Vec<_>>();
                args.push(format!("{} mux", mux_enum_name));
                let args_formatted = Self::format_function_args(&args, false);

                start_block!(
                    out,
                    "[[nodiscard]] static std::expected<{}, CanError> create({}) noexcept",
                    msg_name,
                    args_formatted
                );
                line!(out, "{} msg{{}};", msg_name);
                Self::emit_signal_set_calls(out, "msg", &plain_sigs);

                start_block!(out, "std::visit([&msg](const auto& v)");
                line!(out, "using T = std::decay_t<decltype(v)>;");
                for (mux_value, _) in &muxed_sigs {
                    let variant_class = format!("{}Mux{}", msg_name, mux_value);
                    start_block!(out, "if constexpr (std::is_same_v<T, {}>)", variant_class);
                    line!(out, "msg.set_mux_{}(v);", mux_value);
                    end_block!(out, "");
                }
                end_block_no_close!(out, "}}, mux);");
                empty!(out);
                end_block!(out, "return msg;");
                empty!(out);

                Self::emit_try_from_frame(out, &msg_name);

                Self::emit_signal_getters(out, &plain_sigs, file, config);
                Self::emit_signal_setters(out, &plain_sigs, file, config);

                let mux_layout = &file.signal_layouts[mux_sig.layout.0];
                let mux_raw_type = mux_sig.raw_type.as_cpp_type();
                let mux_extract_fn = Self::detail_extract_fn(mux_layout.byte_order);
                let mux_insert_fn = Self::detail_insert_fn(mux_layout.byte_order);

                // Mux Getter
                start_block!(
                    out,
                    "[[nodiscard]] std::expected<{}, CanError> mux() const noexcept",
                    mux_enum_name
                );
                Self::emit_detail_extract(
                    out,
                    mux_raw_type,
                    "mux_raw",
                    mux_extract_fn,
                    "data_.data()",
                    mux_layout,
                );
                start_block!(out, "switch (mux_raw)");
                for (mux_value, _) in &muxed_sigs {
                    let variant_class = format!("{}Mux{}", msg_name, mux_value);
                    start_block!(out, "case {}:", mux_value);
                    line!(out, "{} inner{{}};", variant_class);
                    line!(out, "std::memcpy(inner.data_.data(), data_.data(), LEN);");
                    end_block!(out, "return inner;");
                }
                end_block!(
                    out,
                    "default: return std::unexpected(CanError::UnknownMuxValue);"
                );
                end_block!(out, "");
                empty!(out);

                for (mux_value, sigs) in &muxed_sigs {
                    let variant_class = format!("{}Mux{}", msg_name, mux_value);
                    start_block!(
                        out,
                        "void set_mux_{}(const {}& value) noexcept",
                        mux_value,
                        variant_class
                    );
                    for sig in sigs {
                        let layout = &file.signal_layouts[sig.layout.0];
                        let copy_fn = Self::detail_copy_fn(layout.byte_order);
                        Self::emit_detail_copy(
                            out,
                            copy_fn,
                            "data_.data()",
                            "value.data_.data()",
                            layout,
                        );
                    }
                    Self::emit_detail_insert(
                        out,
                        mux_raw_type,
                        mux_insert_fn,
                        "data_.data()",
                        mux_layout,
                        &format!("static_cast<{}>({})", mux_raw_type, mux_value),
                    );
                    end_block!(out, "");
                }
                empty!(out);

                Self::emit_encode_method(out);

                Self::emit_data_storage(out, None);

                end_block!(out, "");
                line!(out, "static_assert(GeneratedCanMessage<{}>);", msg_name);
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
            "parse_can(CanId id, std::span<const uint8_t> frame) noexcept"
        );

        start_block!(out, "switch (id.dispatch_key())");

        for msg in messages {
            let name = msg.name.upper_camel();
            line!(out, "case {}::ID.dispatch_key():", name);
            start_block!(out, "");
            line!(out, "auto r = {}::try_from_frame(id, frame);", name);
            line!(out, "if (!r) return std::unexpected(r.error());");
            line!(out, "return CanMsg{{std::move(*r)}};");
            end_block!(out, "");
        }

        end_block!(
            out,
            "default: return std::unexpected(CanError::UnknownFrameId);"
        );
        end_block!(out, "");
    }
}
