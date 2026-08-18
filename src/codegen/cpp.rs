use std::collections::{BTreeMap, BTreeSet};

use heck::ToSnakeCase;

use crate::{
    DbcFile,
    codegen::Generator,
    empty, end_block, end_block_no_close,
    ir::{
        message::{Message, MessageId, MessageSignalClassification},
        signal::Signal,
        signal_layout::{ByteOrder, SignalLayout, ValueType},
        signal_value_enum::SignalValueEnum,
        signal_value_type::{CppType, RawType},
    },
    line, start_block,
};

use crate::codegen::config::{CodegenConfig, CppCodeInjectionPoint};

pub struct CppGen;

pub struct GeneratedCppFile {
    pub file_name: String,
    pub contents: String,
}

fn cpp_code_injections(out: &mut Generator, config: &CodegenConfig, point: CppCodeInjectionPoint) {
    if let Some(injections) = config.cpp_code_injections.get(&point) {
        for injection in injections {
            for line in injection.lines() {
                line!(out, "{}", line);
            }
        }
    }
}

impl CppGen {
    pub fn generate(file: &DbcFile, config: &CodegenConfig) -> String {
        let mut out = Generator::new();

        line!(out, "#pragma once");
        empty!(out);

        Self::common_declarations(&mut out, file, config);

        for message in &file.messages {
            Self::message(&mut out, message, file, config);
        }

        Self::parse_can(&mut out, &file.messages, config);
        if config.generate_tests {
            Self::test_module(&mut out, file, config);
        }
        cpp_code_injections(&mut out, config, CppCodeInjectionPoint::Footer);

        out.into_string()
    }

    pub fn generate_separate(
        file: &DbcFile,
        config: &CodegenConfig,
        output_stem: &str,
    ) -> Vec<GeneratedCppFile> {
        let common_file_name = format!("{}_common.hpp", output_stem);
        let aggregate_file_name = format!("{}.hpp", output_stem);
        let message_file_names = file
            .messages
            .iter()
            .map(|message| Self::message_file_name(output_stem, message))
            .collect::<Vec<_>>();

        let mut files = Vec::with_capacity(file.messages.len() + 2);
        files.push(GeneratedCppFile {
            file_name: common_file_name.clone(),
            contents: Self::generate_common_header(file, config),
        });

        for (message, file_name) in file.messages.iter().zip(&message_file_names) {
            files.push(GeneratedCppFile {
                file_name: file_name.clone(),
                contents: Self::generate_message_header(message, file, config, &common_file_name),
            });
        }

        files.push(GeneratedCppFile {
            file_name: aggregate_file_name,
            contents: Self::generate_aggregate_header(
                file,
                config,
                &common_file_name,
                &message_file_names,
            ),
        });

        files
    }

    fn message_file_name(output_stem: &str, message: &Message) -> String {
        format!(
            "{}_{}.hpp",
            output_stem,
            message.name.upper_camel().to_snake_case()
        )
    }

    fn include_local(out: &mut Generator, file_name: &str) {
        line!(out, "#include \"{}\"", file_name);
    }

    fn generate_common_header(file: &DbcFile, config: &CodegenConfig) -> String {
        let mut out = Generator::new();

        line!(out, "#pragma once");
        empty!(out);

        Self::common_declarations(&mut out, file, config);

        out.into_string()
    }

    fn generate_message_header(
        message: &Message,
        file: &DbcFile,
        config: &CodegenConfig,
        common_file_name: &str,
    ) -> String {
        let mut out = Generator::new();

        line!(out, "#pragma once");
        empty!(out);
        Self::include_local(&mut out, common_file_name);
        empty!(out);

        Self::message(&mut out, message, file, config);

        out.into_string()
    }

    fn generate_aggregate_header(
        file: &DbcFile,
        config: &CodegenConfig,
        common_file_name: &str,
        message_file_names: &[String],
    ) -> String {
        let mut out = Generator::new();

        line!(out, "#pragma once");
        empty!(out);
        Self::include_local(&mut out, common_file_name);
        for file_name in message_file_names {
            Self::include_local(&mut out, file_name);
        }
        empty!(out);

        Self::parse_can(&mut out, &file.messages, config);
        if config.generate_tests {
            Self::test_module(&mut out, file, config);
        }
        cpp_code_injections(&mut out, config, CppCodeInjectionPoint::Footer);

        out.into_string()
    }

    fn common_declarations(out: &mut Generator, file: &DbcFile, config: &CodegenConfig) {
        Self::includes(out, config.generate_tests);
        cpp_code_injections(out, config, CppCodeInjectionPoint::Header);
        Self::errors(out, config);
        Self::can_id(out);
        Self::message_interface(out);
        Self::endian_read_and_write(out);

        let mut emitted_enum_idxs = BTreeSet::new();
        for signal in &file.signals {
            if let Some(idx) = signal.signal_value_enum_idx {
                if emitted_enum_idxs.insert(idx.0) {
                    Self::signal_value_enum(out, signal, &file.signal_value_enums[idx.0], config);
                }
            }
        }
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
            if config.enum_other {
                enum_name
            } else {
                format!("std::expected<{}, CanError>", enum_name)
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

    fn emit_data_storage(
        out: &mut Generator,
        friend_class: Option<&str>,
        config: &CodegenConfig,
        private_injection_point: CppCodeInjectionPoint,
    ) {
        line!(out, "private:");
        cpp_code_injections(out, config, private_injection_point);
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

    fn includes(out: &mut Generator, generate_tests: bool) {
        let mut includes = vec![
            "array",
            "cstddef",
            "cstdint",
            "expected",
            "span",
            "variant",
            "utility",
            "cstring",
            "limits",
            "type_traits",
        ];

        if generate_tests {
            includes.extend(["cmath", "cstdlib"]);
        }

        for include in includes {
            line!(out, "#include <{}>", include);
        }
        empty!(out);
    }

    fn errors(out: &mut Generator, config: &CodegenConfig) {
        const ERRORS: &[&str] = &[
            "UnknownFrameId",
            "UnknownMuxValue",
            "InvalidPayloadSize",
            "InvalidFrameId",
            "ValueOutOfRange",
            "InvalidEnumValue",
        ];

        cpp_code_injections(out, config, CppCodeInjectionPoint::ErrorEnum);
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

        Self::checked_int_math_fns(out);
        Self::extract_le_fn(out);
        Self::extract_be_fn(out);
        Self::insert_le_fn(out);
        Self::insert_be_fn(out);
        Self::copy_le_fn(out);
        Self::copy_be_fn(out);

        line!(out, "}} // namespace detail");
        empty!(out);
    }

    fn checked_int_math_fns(out: &mut Generator) {
        line!(out, "template <typename T>");
        start_block!(
            out,
            "[[nodiscard]] constexpr T saturating_add(T lhs, T rhs) noexcept"
        );
        line!(out, "static_assert(std::is_integral_v<T>);");
        line!(out, "T result{{}};");
        start_block!(out, "if (!__builtin_add_overflow(lhs, rhs, &result))");
        end_block!(out, "return result;");
        start_block!(out, "if constexpr (std::is_unsigned_v<T>)");
        end_block!(out, "return std::numeric_limits<T>::max();");
        end_block!(
            out,
            "return rhs < 0 ? std::numeric_limits<T>::min() : std::numeric_limits<T>::max();"
        );
        empty!(out);

        line!(out, "template <typename T>");
        start_block!(
            out,
            "[[nodiscard]] constexpr T saturating_mul(T lhs, T rhs) noexcept"
        );
        line!(out, "static_assert(std::is_integral_v<T>);");
        line!(out, "T result{{}};");
        start_block!(out, "if (!__builtin_mul_overflow(lhs, rhs, &result))");
        end_block!(out, "return result;");
        start_block!(out, "if constexpr (std::is_unsigned_v<T>)");
        end_block!(out, "return std::numeric_limits<T>::max();");
        end_block!(
            out,
            "return (lhs < 0) == (rhs < 0) ? std::numeric_limits<T>::max() : std::numeric_limits<T>::min();"
        );
        empty!(out);

        line!(out, "template <typename T>");
        line!(out, "[[nodiscard]] constexpr std::expected<T, CanError>");
        start_block!(out, "checked_sub(T lhs, T rhs) noexcept");
        line!(out, "static_assert(std::is_integral_v<T>);");
        line!(out, "T result{{}};");
        start_block!(out, "if (__builtin_sub_overflow(lhs, rhs, &result))");
        end_block!(out, "return std::unexpected(CanError::ValueOutOfRange);");
        end_block!(out, "return result;");
        empty!(out);

        line!(out, "template <typename T>");
        line!(out, "[[nodiscard]] constexpr std::expected<T, CanError>");
        start_block!(out, "checked_rem(T lhs, T rhs) noexcept");
        line!(out, "static_assert(std::is_integral_v<T>);");
        start_block!(out, "if (rhs == 0)");
        end_block!(out, "return std::unexpected(CanError::ValueOutOfRange);");
        start_block!(out, "if constexpr (std::is_signed_v<T>)");
        start_block!(
            out,
            "if (lhs == std::numeric_limits<T>::min() && rhs == static_cast<T>(-1))"
        );
        end_block!(out, "return std::unexpected(CanError::ValueOutOfRange);");
        end_block!(out, "");
        end_block!(out, "return lhs % rhs;");
        empty!(out);

        line!(out, "template <typename T>");
        line!(out, "[[nodiscard]] constexpr std::expected<T, CanError>");
        start_block!(out, "checked_div(T lhs, T rhs) noexcept");
        line!(out, "static_assert(std::is_integral_v<T>);");
        start_block!(out, "if (rhs == 0)");
        end_block!(out, "return std::unexpected(CanError::ValueOutOfRange);");
        start_block!(out, "if constexpr (std::is_signed_v<T>)");
        start_block!(
            out,
            "if (lhs == std::numeric_limits<T>::min() && rhs == static_cast<T>(-1))"
        );
        end_block!(out, "return std::unexpected(CanError::ValueOutOfRange);");
        end_block!(out, "");
        end_block!(out, "return lhs / rhs;");
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

        cpp_code_injections(out, config, CppCodeInjectionPoint::SignalValueEnum);
        start_block!(out, "enum class {} : {}", name, cpp_type);
        for variant in &enum_def.variants {
            line!(out, "{} = {},", variant.description, variant.value);
        }
        end_block!(out, "");
        empty!(out);

        if config.enum_other {
            line!(out, "[[nodiscard]] constexpr {}", name);
            start_block!(
                out,
                "{}_from_raw({} v) noexcept",
                name.to_snake_case(),
                cpp_type
            );
            line!(out, "return static_cast<{}>(v);", name);
            end_block!(out, "");
        } else {
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
                    "return detail::saturating_add<{}>(detail::saturating_mul<{}>(static_cast<{}>({}), static_cast<{}>({})), static_cast<{}>({}));",
                    phys_type,
                    phys_type,
                    phys_type,
                    raw_name,
                    phys_type,
                    layout.factor as i64,
                    phys_type,
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

        if config.allow_unrestricted_ranges && min == max && min == 0.0 {
            return;
        }

        let phys_type = signal.physical_type.as_cpp_type();
        let is_phys_float = phys_type == "float" || phys_type == "double";

        let conditions = if is_phys_float {
            vec![
                format!(
                    "{} < {}",
                    field_name,
                    Self::format_cpp_float(min, phys_type)
                ),
                format!(
                    "{} > {}",
                    field_name,
                    Self::format_cpp_float(max, phys_type)
                ),
            ]
        } else {
            let Some((type_min, type_max)) = signal.physical_type.integer_range_f64() else {
                return;
            };

            let mut conditions = Vec::new();
            if min > type_min {
                conditions.push(format!(
                    "{} < {}",
                    field_name,
                    Self::format_cpp_integer_bound(min)
                ));
            }
            if max < type_max {
                conditions.push(format!(
                    "{} > {}",
                    field_name,
                    Self::format_cpp_integer_bound(max)
                ));
            }
            conditions
        };

        if !conditions.is_empty() {
            line!(
                out,
                "if ({}) return std::unexpected(CanError::ValueOutOfRange);",
                conditions.join(" || ")
            );
        }
    }

    fn format_cpp_integer_bound(value: f64) -> String {
        format!("{:.0}", value.trunc())
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
                let factor = layout.factor as i64;
                line!(
                    out,
                    "const auto {}_shifted = detail::checked_sub<{}>({}, static_cast<{}>({}));",
                    field_name,
                    phys_type,
                    field_name,
                    phys_type,
                    layout.offset as i64
                );
                line!(
                    out,
                    "if (!{}_shifted) return std::unexpected({}_shifted.error());",
                    field_name,
                    field_name
                );
                if factor.abs() != 1 {
                    line!(
                        out,
                        "const auto {}_remainder = detail::checked_rem<{}>(*{}_shifted, static_cast<{}>({}));",
                        field_name,
                        phys_type,
                        field_name,
                        phys_type,
                        factor
                    );
                    line!(
                        out,
                        "if (!{}_remainder) return std::unexpected({}_remainder.error());",
                        field_name,
                        field_name
                    );
                    line!(
                        out,
                        "if (*{}_remainder != static_cast<{}>(0)) return std::unexpected(CanError::ValueOutOfRange);",
                        field_name,
                        phys_type
                    );
                }
                line!(
                    out,
                    "const auto {}_raw = detail::checked_div<{}>(*{}_shifted, static_cast<{}>({}));",
                    field_name,
                    phys_type,
                    field_name,
                    phys_type,
                    factor
                );
                line!(
                    out,
                    "if (!{}_raw) return std::unexpected({}_raw.error());",
                    field_name,
                    field_name
                );
                Self::emit_detail_insert(
                    out,
                    raw_type,
                    insert_fn,
                    data_expr,
                    layout,
                    &format!("static_cast<{}>(*{}_raw)", raw_type, field_name),
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

        cpp_code_injections(out, config, CppCodeInjectionPoint::MuxVariantClass);
        start_block!(out, "class {}", class_name);

        line!(out, "public:");
        cpp_code_injections(out, config, CppCodeInjectionPoint::MuxVariantClassPublic);

        Self::emit_len_constant(out, msg.size);
        empty!(out);

        Self::emit_create_method(out, &class_name, signals, file);

        Self::emit_signal_getters(out, signals, file, config);
        Self::emit_signal_setters(out, signals, file, config);

        Self::emit_encode_method(out);

        // Required for multiplex setter
        Self::emit_data_storage(
            out,
            Some(&msg.name.upper_camel()),
            config,
            CppCodeInjectionPoint::MuxVariantClassPrivate,
        );

        end_block!(out, "");
        empty!(out);
    }

    fn message(out: &mut Generator, msg: &Message, file: &DbcFile, config: &CodegenConfig) {
        match msg.classify_signals(&file.signals) {
            MessageSignalClassification::Plain { signals } => {
                let sigs: Vec<&Signal> = signals.iter().map(|idx| &file.signals[idx.0]).collect();
                let msg_name = msg.name.upper_camel();

                Self::emit_message_doc(out, msg);
                cpp_code_injections(out, config, CppCodeInjectionPoint::MessageClass);
                start_block!(out, "class {}", msg_name);
                line!(out, "public:");
                cpp_code_injections(out, config, CppCodeInjectionPoint::MessageClassPublic);

                Self::emit_message_id(out, msg);
                Self::emit_len_constant(out, msg.size);
                empty!(out);

                Self::emit_create_method(out, &msg_name, &sigs, file);

                Self::emit_try_from_frame(out, &msg_name);

                Self::emit_signal_getters(out, &sigs, file, config);
                Self::emit_signal_setters(out, &sigs, file, config);

                Self::emit_encode_method(out);

                Self::emit_data_storage(
                    out,
                    None,
                    config,
                    CppCodeInjectionPoint::MessageClassPrivate,
                );

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
                cpp_code_injections(out, config, CppCodeInjectionPoint::MuxVariant);
                line!(
                    out,
                    "using {} = std::variant<{}>;",
                    mux_enum_name,
                    variant_types
                );
                empty!(out);

                Self::emit_message_doc(out, msg);
                cpp_code_injections(out, config, CppCodeInjectionPoint::MessageClass);
                start_block!(out, "class {}", msg_name);
                line!(out, "public:");
                cpp_code_injections(out, config, CppCodeInjectionPoint::MessageClassPublic);

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

                Self::emit_data_storage(
                    out,
                    None,
                    config,
                    CppCodeInjectionPoint::MessageClassPrivate,
                );

                end_block!(out, "");
                line!(out, "static_assert(GeneratedCanMessage<{}>);", msg_name);
                empty!(out);
            }
        }
    }

    fn test_module(out: &mut Generator, file: &DbcFile, config: &CodegenConfig) {
        empty!(out);
        line!(out, "namespace generated_tests {{");
        empty!(out);

        Self::emit_test_constants(out, file);
        Self::emit_test_helpers(out);
        Self::parse_can_error_test(out);

        for msg in &file.messages {
            match msg.classify_signals(&file.signals) {
                MessageSignalClassification::Plain { signals } => {
                    let sigs: Vec<&Signal> =
                        signals.iter().map(|idx| &file.signals[idx.0]).collect();
                    Self::plain_message_test(out, msg, &sigs, file, config);
                }
                MessageSignalClassification::Multiplexed {
                    plain,
                    mux_signal,
                    muxed,
                } => {
                    let plain_sigs: Vec<&Signal> =
                        plain.iter().map(|idx| &file.signals[idx.0]).collect();
                    let mux_sig = &file.signals[mux_signal.0];
                    let muxed_sigs: BTreeMap<u64, Vec<&Signal>> = muxed
                        .iter()
                        .map(|(v, idxs)| {
                            (*v, idxs.iter().map(|idx| &file.signals[idx.0]).collect())
                        })
                        .collect();
                    Self::multiplexed_message_test(
                        out,
                        msg,
                        &plain_sigs,
                        mux_sig,
                        &muxed_sigs,
                        file,
                        config,
                    );
                }
            }
        }

        start_block!(out, "inline void run_all()");
        line!(out, "test_parse_can_errors();");
        for msg in &file.messages {
            line!(out, "{}();", Self::test_fn_name(msg));
        }
        end_block!(out, "");
        empty!(out);

        line!(out, "}} // namespace generated_tests");
    }

    fn emit_test_constants(out: &mut Generator, file: &DbcFile) {
        line!(
            out,
            "inline constexpr CanId UNKNOWN_FRAME_ID = {};",
            Self::format_can_id_expr(&Self::test_unknown_frame_id(&file.messages))
        );
        empty!(out);
    }

    fn emit_test_helpers(out: &mut Generator) {
        start_block!(out, "inline void expect(bool condition)");
        start_block!(out, "if (!condition)");
        line!(out, "std::abort();");
        end_block!(out, "");
        end_block!(out, "");
        empty!(out);

        line!(out, "template <typename Actual, typename Expected>");
        start_block!(
            out,
            "inline void expect_equal(const Actual& actual, const Expected& expected)"
        );
        line!(out, "expect(actual == expected);");
        end_block!(out, "");
        empty!(out);

        line!(out, "template <typename Actual, typename Expected>");
        start_block!(
            out,
            "inline void expect_near(Actual actual, Expected expected)"
        );
        line!(out, "const double a = static_cast<double>(actual);");
        line!(out, "const double e = static_cast<double>(expected);");
        line!(out, "double tolerance = std::fabs(e) * 0.0001;");
        line!(out, "if (tolerance < 0.0001) tolerance = 0.0001;");
        line!(out, "expect(std::fabs(a - e) <= tolerance);");
        end_block!(out, "");
        empty!(out);

        line!(out, "template <typename T>");
        start_block!(
            out,
            "inline void expect_error(const std::expected<T, CanError>& result, CanError expected)"
        );
        line!(out, "expect(!result.has_value());");
        line!(out, "expect_equal(result.error(), expected);");
        end_block!(out, "");
        empty!(out);
    }

    fn parse_can_error_test(out: &mut Generator) {
        start_block!(out, "inline void test_parse_can_errors()");
        line!(out, "const std::array<uint8_t, 8> frame{{}};");
        line!(
            out,
            "auto unknown_id_result = parse_can(UNKNOWN_FRAME_ID, frame);"
        );
        line!(
            out,
            "expect_error(unknown_id_result, CanError::UnknownFrameId);"
        );
        end_block!(out, "");
        empty!(out);
    }

    fn plain_message_test(
        out: &mut Generator,
        msg: &Message,
        signals: &[&Signal],
        file: &DbcFile,
        config: &CodegenConfig,
    ) {
        let msg_name = msg.name.upper_camel();
        let test_name = Self::test_fn_name(msg);

        start_block!(out, "inline void {}()", test_name);
        Self::emit_test_value_decls(out, signals, file, config, "value", 0);

        let constructor_args = Self::test_vars(signals, "value").join(", ");
        line!(
            out,
            "auto msg_result = {}::create({});",
            msg_name,
            constructor_args
        );
        line!(out, "expect(msg_result.has_value());");
        line!(out, "auto msg = *msg_result;");
        Self::emit_test_getter_assertions(out, "msg", signals, "value", file, config);

        Self::emit_test_value_decls(out, signals, file, config, "next_value", 1);
        Self::emit_test_setter_calls(out, "msg", signals, "next_value");
        Self::emit_test_getter_assertions(out, "msg", signals, "next_value", file, config);
        Self::emit_test_create_range_error_assertions(
            out,
            &msg_name,
            signals,
            file,
            config,
            "next_value",
            &[],
        );
        Self::emit_test_setter_range_error_assertions(out, "msg", signals, file, config);

        line!(out, "const auto encoded = msg.encode();");
        line!(
            out,
            "auto frame_result = {}::try_from_frame({}::ID, encoded);",
            msg_name,
            msg_name
        );
        line!(out, "expect(frame_result.has_value());");
        line!(out, "auto frame_msg = *frame_result;");
        Self::emit_test_getter_assertions(out, "frame_msg", signals, "next_value", file, config);

        line!(
            out,
            "auto parsed_result = parse_can({}::ID, encoded);",
            msg_name
        );
        line!(out, "expect(parsed_result.has_value());");
        line!(
            out,
            "expect(std::get_if<{}>(&*parsed_result) != nullptr);",
            msg_name
        );
        Self::emit_test_frame_error_assertions(out, &msg_name);
        Self::emit_test_invalid_enum_payload_assertions(out, &msg_name, signals, file, config);

        end_block!(out, "");
        empty!(out);
    }

    fn multiplexed_message_test(
        out: &mut Generator,
        msg: &Message,
        plain: &[&Signal],
        mux_signal: &Signal,
        muxed: &BTreeMap<u64, Vec<&Signal>>,
        file: &DbcFile,
        config: &CodegenConfig,
    ) {
        let msg_name = msg.name.upper_camel();
        let mux_enum_name = format!("{}Mux", msg_name);
        let test_name = Self::test_fn_name(msg);
        let mux_entries = muxed.iter().collect::<Vec<_>>();

        start_block!(out, "inline void {}()", test_name);
        Self::emit_test_value_decls(out, plain, file, config, "value", 0);

        for (entry_idx, (mux_value, sigs)) in mux_entries.iter().enumerate() {
            let next_entry_idx = if mux_entries.len() > 1 {
                (entry_idx + 1) % mux_entries.len()
            } else {
                entry_idx
            };
            let (next_mux_value, next_sigs) = mux_entries[next_entry_idx];

            let variant_class = format!("{}Mux{}", msg_name, mux_value);
            let next_variant_class = format!("{}Mux{}", msg_name, next_mux_value);
            let mux_setter = format!("set_mux_{}", next_mux_value);

            start_block!(out, "");

            Self::emit_test_value_decls(out, sigs, file, config, "value", 0);
            let mux_constructor_args = Self::test_vars(sigs, "value").join(", ");
            line!(
                out,
                "auto mux_msg_result = {}::create({});",
                variant_class,
                mux_constructor_args
            );
            line!(out, "expect(mux_msg_result.has_value());");
            line!(out, "auto mux_value = *mux_msg_result;");

            let mut msg_constructor_args = Self::test_vars(plain, "value");
            msg_constructor_args.push(format!("{}{{mux_value}}", mux_enum_name));
            line!(
                out,
                "auto msg_result = {}::create({});",
                msg_name,
                msg_constructor_args.join(", ")
            );
            line!(out, "expect(msg_result.has_value());");
            line!(out, "auto msg = *msg_result;");
            Self::emit_test_getter_assertions(out, "msg", plain, "value", file, config);

            line!(out, "auto mux_result = msg.mux();");
            line!(out, "expect(mux_result.has_value());");
            line!(
                out,
                "const auto* mux_msg_ptr = std::get_if<{}>(&*mux_result);",
                variant_class
            );
            line!(out, "expect(mux_msg_ptr != nullptr);");
            line!(out, "auto mux_msg = *mux_msg_ptr;");
            Self::emit_test_getter_assertions(out, "mux_msg", sigs, "value", file, config);

            Self::emit_test_value_decls(out, plain, file, config, "next_value", 1);
            Self::emit_test_setter_calls(out, "msg", plain, "next_value");
            Self::emit_test_getter_assertions(out, "msg", plain, "next_value", file, config);
            if entry_idx == 0 {
                Self::emit_test_create_range_error_assertions(
                    out,
                    &msg_name,
                    plain,
                    file,
                    config,
                    "next_value",
                    &[format!("{}{{mux_value}}", mux_enum_name)],
                );
                Self::emit_test_setter_range_error_assertions(out, "msg", plain, file, config);
            }

            Self::emit_test_value_decls(out, sigs, file, config, "next_value", 1);
            Self::emit_test_setter_calls(out, "mux_msg", sigs, "next_value");
            Self::emit_test_getter_assertions(out, "mux_msg", sigs, "next_value", file, config);
            Self::emit_test_create_range_error_assertions(
                out,
                &variant_class,
                sigs,
                file,
                config,
                "next_value",
                &[],
            );
            Self::emit_test_setter_range_error_assertions(out, "mux_msg", sigs, file, config);

            Self::emit_test_value_decls(out, next_sigs, file, config, "switch_value", 2);
            let next_mux_constructor_args = Self::test_vars(next_sigs, "switch_value").join(", ");
            line!(
                out,
                "auto next_mux_msg_result = {}::create({});",
                next_variant_class,
                next_mux_constructor_args
            );
            line!(out, "expect(next_mux_msg_result.has_value());");
            line!(out, "auto next_mux_value = *next_mux_msg_result;");
            line!(out, "msg.{}(next_mux_value);", mux_setter);

            line!(out, "auto switched_mux_result = msg.mux();");
            line!(out, "expect(switched_mux_result.has_value());");
            line!(
                out,
                "const auto* switched_mux_ptr = std::get_if<{}>(&*switched_mux_result);",
                next_variant_class
            );
            line!(out, "expect(switched_mux_ptr != nullptr);");
            line!(out, "auto switched_mux_msg = *switched_mux_ptr;");
            Self::emit_test_getter_assertions(
                out,
                "switched_mux_msg",
                next_sigs,
                "switch_value",
                file,
                config,
            );

            Self::emit_test_value_decls(out, next_sigs, file, config, "switch_next_value", 3);
            Self::emit_test_setter_calls(out, "switched_mux_msg", next_sigs, "switch_next_value");
            Self::emit_test_getter_assertions(
                out,
                "switched_mux_msg",
                next_sigs,
                "switch_next_value",
                file,
                config,
            );

            line!(out, "msg.{}(switched_mux_msg);", mux_setter);
            line!(out, "auto updated_mux_result = msg.mux();");
            line!(out, "expect(updated_mux_result.has_value());");
            line!(
                out,
                "const auto* updated_mux_ptr = std::get_if<{}>(&*updated_mux_result);",
                next_variant_class
            );
            line!(out, "expect(updated_mux_ptr != nullptr);");
            line!(out, "auto updated_mux_msg = *updated_mux_ptr;");
            Self::emit_test_getter_assertions(
                out,
                "updated_mux_msg",
                next_sigs,
                "switch_next_value",
                file,
                config,
            );
            Self::emit_test_getter_assertions(out, "msg", plain, "next_value", file, config);

            line!(out, "const auto encoded = msg.encode();");
            line!(
                out,
                "auto frame_result = {}::try_from_frame({}::ID, encoded);",
                msg_name,
                msg_name
            );
            line!(out, "expect(frame_result.has_value());");
            line!(out, "auto frame_msg = *frame_result;");
            Self::emit_test_getter_assertions(out, "frame_msg", plain, "next_value", file, config);
            line!(out, "auto frame_mux_result = frame_msg.mux();");
            line!(out, "expect(frame_mux_result.has_value());");
            line!(
                out,
                "const auto* frame_mux_ptr = std::get_if<{}>(&*frame_mux_result);",
                next_variant_class
            );
            line!(out, "expect(frame_mux_ptr != nullptr);");
            line!(out, "auto frame_mux_msg = *frame_mux_ptr;");
            Self::emit_test_getter_assertions(
                out,
                "frame_mux_msg",
                next_sigs,
                "switch_next_value",
                file,
                config,
            );

            line!(
                out,
                "auto parsed_result = parse_can({}::ID, encoded);",
                msg_name
            );
            line!(out, "expect(parsed_result.has_value());");
            line!(
                out,
                "expect(std::get_if<{}>(&*parsed_result) != nullptr);",
                msg_name
            );
            Self::emit_test_frame_error_assertions(out, &msg_name);
            Self::emit_test_invalid_mux_payload_assertion(out, &msg_name, mux_signal, muxed, file);
            Self::emit_test_invalid_enum_payload_assertions(out, &msg_name, plain, file, config);
            Self::emit_test_invalid_mux_enum_payload_assertions(
                out,
                &msg_name,
                &next_variant_class,
                next_sigs,
                file,
                config,
            );

            end_block!(out, "");
        }

        end_block!(out, "");
        empty!(out);
    }

    fn emit_test_frame_error_assertions(out: &mut Generator, msg_name: &str) {
        line!(
            out,
            "auto wrong_id_result = {}::try_from_frame(UNKNOWN_FRAME_ID, encoded);",
            msg_name
        );
        line!(
            out,
            "expect_error(wrong_id_result, CanError::InvalidFrameId);"
        );
        start_block!(out, "if constexpr ({}::LEN > 0)", msg_name);
        line!(
            out,
            "const std::span<const uint8_t> short_frame{{encoded.data(), {}::LEN - 1}};",
            msg_name
        );
        line!(
            out,
            "auto short_frame_result = {}::try_from_frame({}::ID, short_frame);",
            msg_name,
            msg_name
        );
        line!(
            out,
            "expect_error(short_frame_result, CanError::InvalidPayloadSize);"
        );
        line!(
            out,
            "auto short_parse_result = parse_can({}::ID, short_frame);",
            msg_name
        );
        line!(
            out,
            "expect_error(short_parse_result, CanError::InvalidPayloadSize);"
        );
        end_block!(out, "");
    }

    fn emit_test_create_range_error_assertions(
        out: &mut Generator,
        type_name: &str,
        signals: &[&Signal],
        file: &DbcFile,
        config: &CodegenConfig,
        valid_suffix: &str,
        trailing_args: &[String],
    ) {
        for (bad_idx, signal) in signals.iter().enumerate() {
            let Some(invalid_value) = Self::out_of_range_test_value_expr(signal, file, config)
            else {
                continue;
            };

            let field_name = signal.name.raw.to_snake_case();
            let invalid_var = format!("{}_out_of_range", field_name);
            let mut constructor_args = Self::test_vars(signals, valid_suffix);
            constructor_args[bad_idx] = invalid_var.clone();
            constructor_args.extend(trailing_args.iter().cloned());

            start_block!(out, "");
            line!(out, "const auto {} = {};", invalid_var, invalid_value);
            line!(
                out,
                "auto create_{}_out_of_range_result = {}::create({});",
                field_name,
                type_name,
                constructor_args.join(", ")
            );
            line!(
                out,
                "expect_error(create_{}_out_of_range_result, CanError::ValueOutOfRange);",
                field_name
            );
            end_block!(out, "");
        }
    }

    fn emit_test_setter_range_error_assertions(
        out: &mut Generator,
        receiver: &str,
        signals: &[&Signal],
        file: &DbcFile,
        config: &CodegenConfig,
    ) {
        for signal in signals {
            let Some(invalid_value) = Self::out_of_range_test_value_expr(signal, file, config)
            else {
                continue;
            };

            let field_name = signal.name.raw.to_snake_case();
            let invalid_var = format!("{}_out_of_range", field_name);

            start_block!(out, "");
            line!(out, "const auto {} = {};", invalid_var, invalid_value);
            line!(
                out,
                "auto set_{}_out_of_range_result = {}.set_{}({});",
                field_name,
                receiver,
                field_name,
                invalid_var
            );
            line!(
                out,
                "expect_error(set_{}_out_of_range_result, CanError::ValueOutOfRange);",
                field_name
            );
            end_block!(out, "");
        }
    }

    fn emit_test_invalid_enum_payload_assertions(
        out: &mut Generator,
        msg_name: &str,
        signals: &[&Signal],
        file: &DbcFile,
        config: &CodegenConfig,
    ) {
        if config.enum_other {
            return;
        }

        for signal in signals {
            let Some(invalid_raw) = Self::invalid_enum_raw_value(signal, file) else {
                continue;
            };

            let layout = &file.signal_layouts[signal.layout.0];
            let raw_type = signal.raw_type.as_cpp_type();
            let insert_fn = Self::detail_insert_fn(layout.byte_order);
            let field_name = signal.name.raw.to_snake_case();

            start_block!(out, "");
            line!(out, "auto invalid_enum_frame = encoded;");
            Self::emit_detail_insert(
                out,
                raw_type,
                insert_fn,
                "invalid_enum_frame.data()",
                layout,
                &format!("static_cast<{}>({})", raw_type, invalid_raw),
            );
            line!(
                out,
                "auto invalid_enum_frame_result = {}::try_from_frame({}::ID, invalid_enum_frame);",
                msg_name,
                msg_name
            );
            line!(out, "expect(invalid_enum_frame_result.has_value());");
            line!(out, "auto invalid_enum_msg = *invalid_enum_frame_result;");
            line!(
                out,
                "const auto invalid_enum_result = invalid_enum_msg.{}();",
                field_name
            );
            line!(
                out,
                "expect_error(invalid_enum_result, CanError::InvalidEnumValue);"
            );
            line!(
                out,
                "auto invalid_enum_parsed_result = parse_can({}::ID, invalid_enum_frame);",
                msg_name
            );
            line!(out, "expect(invalid_enum_parsed_result.has_value());");
            line!(
                out,
                "const auto* invalid_enum_parsed_msg = std::get_if<{}>(&*invalid_enum_parsed_result);",
                msg_name
            );
            line!(out, "expect(invalid_enum_parsed_msg != nullptr);");
            line!(
                out,
                "const auto invalid_enum_parsed_signal_result = invalid_enum_parsed_msg->{}();",
                field_name
            );
            line!(
                out,
                "expect_error(invalid_enum_parsed_signal_result, CanError::InvalidEnumValue);"
            );
            end_block!(out, "");
        }
    }

    fn emit_test_invalid_mux_enum_payload_assertions(
        out: &mut Generator,
        msg_name: &str,
        variant_class: &str,
        signals: &[&Signal],
        file: &DbcFile,
        config: &CodegenConfig,
    ) {
        if config.enum_other {
            return;
        }

        for signal in signals {
            let Some(invalid_raw) = Self::invalid_enum_raw_value(signal, file) else {
                continue;
            };

            let layout = &file.signal_layouts[signal.layout.0];
            let raw_type = signal.raw_type.as_cpp_type();
            let insert_fn = Self::detail_insert_fn(layout.byte_order);
            let field_name = signal.name.raw.to_snake_case();

            start_block!(out, "");
            line!(out, "auto invalid_enum_frame = encoded;");
            Self::emit_detail_insert(
                out,
                raw_type,
                insert_fn,
                "invalid_enum_frame.data()",
                layout,
                &format!("static_cast<{}>({})", raw_type, invalid_raw),
            );
            line!(
                out,
                "auto invalid_enum_frame_result = {}::try_from_frame({}::ID, invalid_enum_frame);",
                msg_name,
                msg_name
            );
            line!(out, "expect(invalid_enum_frame_result.has_value());");
            line!(out, "auto invalid_enum_msg = *invalid_enum_frame_result;");
            line!(
                out,
                "auto invalid_enum_mux_result = invalid_enum_msg.mux();"
            );
            line!(out, "expect(invalid_enum_mux_result.has_value());");
            line!(
                out,
                "const auto* invalid_enum_mux_msg = std::get_if<{}>(&*invalid_enum_mux_result);",
                variant_class
            );
            line!(out, "expect(invalid_enum_mux_msg != nullptr);");
            line!(
                out,
                "const auto invalid_enum_result = invalid_enum_mux_msg->{}();",
                field_name
            );
            line!(
                out,
                "expect_error(invalid_enum_result, CanError::InvalidEnumValue);"
            );
            line!(
                out,
                "auto invalid_enum_parsed_result = parse_can({}::ID, invalid_enum_frame);",
                msg_name
            );
            line!(out, "expect(invalid_enum_parsed_result.has_value());");
            line!(
                out,
                "const auto* invalid_enum_parsed_msg = std::get_if<{}>(&*invalid_enum_parsed_result);",
                msg_name
            );
            line!(out, "expect(invalid_enum_parsed_msg != nullptr);");
            line!(
                out,
                "auto invalid_enum_parsed_mux_result = invalid_enum_parsed_msg->mux();"
            );
            line!(out, "expect(invalid_enum_parsed_mux_result.has_value());");
            line!(
                out,
                "const auto* invalid_enum_parsed_mux_msg = std::get_if<{}>(&*invalid_enum_parsed_mux_result);",
                variant_class
            );
            line!(out, "expect(invalid_enum_parsed_mux_msg != nullptr);");
            line!(
                out,
                "const auto invalid_enum_parsed_signal_result = invalid_enum_parsed_mux_msg->{}();",
                field_name
            );
            line!(
                out,
                "expect_error(invalid_enum_parsed_signal_result, CanError::InvalidEnumValue);"
            );
            end_block!(out, "");
        }
    }

    fn emit_test_invalid_mux_payload_assertion(
        out: &mut Generator,
        msg_name: &str,
        mux_signal: &Signal,
        muxed: &BTreeMap<u64, Vec<&Signal>>,
        file: &DbcFile,
    ) {
        let Some(invalid_raw) = Self::invalid_mux_raw_value(mux_signal, muxed, file) else {
            return;
        };

        let layout = &file.signal_layouts[mux_signal.layout.0];
        let raw_type = mux_signal.raw_type.as_cpp_type();
        let insert_fn = Self::detail_insert_fn(layout.byte_order);

        start_block!(out, "");
        line!(out, "auto invalid_mux_frame = encoded;");
        Self::emit_detail_insert(
            out,
            raw_type,
            insert_fn,
            "invalid_mux_frame.data()",
            layout,
            &format!("static_cast<{}>({})", raw_type, invalid_raw),
        );
        line!(
            out,
            "auto invalid_mux_frame_result = {}::try_from_frame({}::ID, invalid_mux_frame);",
            msg_name,
            msg_name
        );
        line!(out, "expect(invalid_mux_frame_result.has_value());");
        line!(out, "auto invalid_mux_msg = *invalid_mux_frame_result;");
        line!(out, "auto invalid_mux_result = invalid_mux_msg.mux();");
        line!(
            out,
            "expect_error(invalid_mux_result, CanError::UnknownMuxValue);"
        );
        line!(
            out,
            "auto invalid_mux_parsed_result = parse_can({}::ID, invalid_mux_frame);",
            msg_name
        );
        line!(out, "expect(invalid_mux_parsed_result.has_value());");
        line!(
            out,
            "const auto* invalid_mux_parsed_msg = std::get_if<{}>(&*invalid_mux_parsed_result);",
            msg_name
        );
        line!(out, "expect(invalid_mux_parsed_msg != nullptr);");
        line!(
            out,
            "auto invalid_mux_parsed_mux_result = invalid_mux_parsed_msg->mux();"
        );
        line!(
            out,
            "expect_error(invalid_mux_parsed_mux_result, CanError::UnknownMuxValue);"
        );
        end_block!(out, "");
    }

    fn emit_test_value_decls(
        out: &mut Generator,
        signals: &[&Signal],
        file: &DbcFile,
        config: &CodegenConfig,
        suffix: &str,
        ordinal: usize,
    ) {
        for signal in signals {
            line!(
                out,
                "const auto {} = {};",
                Self::test_var(signal, suffix),
                Self::test_value_expr(signal, file, config, ordinal)
            );
        }
    }

    fn emit_test_setter_calls(
        out: &mut Generator,
        receiver: &str,
        signals: &[&Signal],
        value_suffix: &str,
    ) {
        for signal in signals {
            let field_name = signal.name.raw.to_snake_case();
            let value = Self::test_var(signal, value_suffix);
            let result = format!("set_{}_{}_result", field_name, value_suffix);
            line!(
                out,
                "auto {} = {}.set_{}({});",
                result,
                receiver,
                field_name,
                value
            );
            line!(out, "expect({}.has_value());", result);
        }
    }

    fn emit_test_getter_assertions(
        out: &mut Generator,
        receiver: &str,
        signals: &[&Signal],
        expected_suffix: &str,
        _file: &DbcFile,
        config: &CodegenConfig,
    ) {
        for signal in signals {
            let field_name = signal.name.raw.to_snake_case();
            let expected = Self::test_var(signal, expected_suffix);
            let call = format!("{}.{}()", receiver, field_name);

            if signal.signal_value_enum_idx.is_some() && !config.enum_other {
                let actual = format!(
                    "actual_{}_{}_{}",
                    Self::test_ident_fragment(receiver),
                    field_name,
                    expected_suffix
                );
                line!(out, "const auto {} = {};", actual, call);
                line!(out, "expect({}.has_value());", actual);
                line!(out, "expect_equal(*{}, {});", actual, expected);
            } else if Self::is_phys_float(signal.physical_type.as_cpp_type()) {
                line!(out, "expect_near({}, {});", call, expected);
            } else {
                line!(out, "expect_equal({}, {});", call, expected);
            }
        }
    }

    fn test_fn_name(msg: &Message) -> String {
        format!("test_{}", msg.name.snake_case())
    }

    fn test_var(signal: &Signal, suffix: &str) -> String {
        format!("{}_{}", signal.name.raw.to_snake_case(), suffix)
    }

    fn test_ident_fragment(value: &str) -> String {
        value
            .chars()
            .map(|ch| {
                if ch == '_' || ch.is_ascii_alphanumeric() {
                    ch
                } else {
                    '_'
                }
            })
            .collect()
    }

    fn test_vars(signals: &[&Signal], suffix: &str) -> Vec<String> {
        signals
            .iter()
            .map(|signal| Self::test_var(signal, suffix))
            .collect()
    }

    fn test_value_expr(
        signal: &Signal,
        file: &DbcFile,
        config: &CodegenConfig,
        ordinal: usize,
    ) -> String {
        if let Some(idx) = signal.signal_value_enum_idx {
            let enum_def = &file.signal_value_enums[idx.0];
            let enum_name = enum_def.name.upper_camel();
            if !enum_def.variants.is_empty() {
                let variant = &enum_def.variants[ordinal % enum_def.variants.len()];
                return format!("{}::{}", enum_name, variant.description);
            }

            return format!("static_cast<{}>(0)", enum_name);
        }

        if Self::is_bool_signal(signal, file) {
            return if ordinal % 2 == 0 { "false" } else { "true" }.to_string();
        }

        let layout = &file.signal_layouts[signal.layout.0];
        let phys_type = signal.physical_type.as_cpp_type();

        if let Some(value) =
            Self::declared_physical_integer_test_value_expr(signal, file, config, ordinal)
        {
            return value;
        }

        if Self::is_raw_float(&signal.raw_type) {
            let value = Self::raw_float_test_value(layout, config, ordinal);
            return Self::format_cpp_float(value, phys_type);
        }

        let active_range_check = Self::test_range_check_active(signal, file, config);
        let (low, high) = Self::test_raw_interval(layout, active_range_check);
        let raw = Self::choose_test_raw(low, high, ordinal, active_range_check);
        let physical = raw as f64 * layout.factor + layout.offset;

        if Self::is_phys_float(phys_type) {
            Self::format_cpp_float(physical, phys_type)
        } else {
            let value = if layout.factor.fract() == 0.0 && layout.offset.fract() == 0.0 {
                raw.saturating_mul(layout.factor as i128)
                    .saturating_add(layout.offset as i128)
            } else {
                physical as i128
            };

            format!(
                "static_cast<{}>({})",
                Self::signal_cpp_param_type(signal, file),
                value
            )
        }
    }

    fn declared_physical_integer_test_value_expr(
        signal: &Signal,
        file: &DbcFile,
        config: &CodegenConfig,
        ordinal: usize,
    ) -> Option<String> {
        if signal.signal_value_enum_idx.is_some()
            || Self::is_bool_signal(signal, file)
            || Self::is_raw_float(&signal.raw_type)
            || Self::is_phys_float(signal.physical_type.as_cpp_type())
            || !Self::test_range_check_active(signal, file, config)
        {
            return None;
        }

        let layout = &file.signal_layouts[signal.layout.0];
        if !layout.min.is_finite() || !layout.max.is_finite() {
            return None;
        }

        let span = layout.max - layout.min;
        if !(1.0..=256.0).contains(&span) {
            return None;
        }

        let candidates = [
            layout.min + 1.0,
            layout.max - 1.0,
            (layout.min + layout.max) / 2.0,
            layout.min,
            layout.max,
        ];
        let value = candidates
            .into_iter()
            .filter(|value| value.is_finite() && value.fract() == 0.0)
            .filter(|value| *value >= layout.min && *value <= layout.max)
            .nth(ordinal % candidates.len())?;

        Some(format!(
            "static_cast<{}>({})",
            Self::signal_cpp_param_type(signal, file),
            value as i128
        ))
    }

    fn out_of_range_test_value_expr(
        signal: &Signal,
        file: &DbcFile,
        config: &CodegenConfig,
    ) -> Option<String> {
        if signal.signal_value_enum_idx.is_some() || Self::is_bool_signal(signal, file) {
            return None;
        }

        if !Self::test_range_check_active(signal, file, config) {
            return None;
        }

        let layout = &file.signal_layouts[signal.layout.0];
        let phys_type = signal.physical_type.as_cpp_type();

        if Self::is_phys_float(phys_type) {
            let min = layout.min;
            let max = layout.max;
            let type_min = if phys_type == "float" {
                f32::MIN as f64
            } else {
                f64::MIN
            };
            let type_max = if phys_type == "float" {
                f32::MAX as f64
            } else {
                f64::MAX
            };
            let range = (max - min).abs();
            let step = if range.is_finite() {
                (range * 0.5).max(1.0)
            } else {
                1.0
            };

            [max + step, min - step, max + 1.0, min - 1.0]
                .into_iter()
                .find(|candidate| {
                    candidate.is_finite()
                        && *candidate >= type_min
                        && *candidate <= type_max
                        && (*candidate < min || *candidate > max)
                })
                .map(|candidate| Self::format_cpp_float(candidate, phys_type))
        } else {
            let min = layout.min as i128;
            let max = layout.max as i128;
            let type_min = signal.physical_type.min_value_f64() as i128;
            let type_max = signal.physical_type.max_value_f64() as i128;

            let candidate = max
                .checked_add(1)
                .filter(|candidate| *candidate <= type_max)
                .or_else(|| {
                    min.checked_sub(1)
                        .filter(|candidate| *candidate >= type_min)
                })?;

            Some(format!(
                "static_cast<{}>({})",
                Self::signal_cpp_param_type(signal, file),
                candidate
            ))
        }
    }

    fn test_range_check_active(signal: &Signal, file: &DbcFile, config: &CodegenConfig) -> bool {
        if signal.signal_value_enum_idx.is_some() {
            return false;
        }

        let layout = &file.signal_layouts[signal.layout.0];
        if layout.size == 1 {
            return false;
        }

        !(config.allow_unrestricted_ranges && layout.min == 0.0 && layout.max == 0.0)
    }

    fn raw_float_test_value(layout: &SignalLayout, config: &CodegenConfig, ordinal: usize) -> f64 {
        let active_range_check =
            !(config.allow_unrestricted_ranges && layout.min == 0.0 && layout.max == 0.0);

        if active_range_check {
            match ordinal % 3 {
                0 => layout.min,
                1 => layout.max,
                _ => (layout.min + layout.max) / 2.0,
            }
        } else {
            match ordinal % 5 {
                0 => 0.0,
                1 => 1.0,
                2 => -1.0,
                3 => 2.0,
                _ => -2.0,
            }
        }
    }

    fn test_raw_interval(layout: &SignalLayout, active_range_check: bool) -> (i128, i128) {
        let (raw_min, raw_max) = Self::integer_raw_range(layout);

        if !active_range_check {
            return (raw_min, raw_max);
        }

        let raw_a = (layout.min - layout.offset) / layout.factor;
        let raw_b = (layout.max - layout.offset) / layout.factor;
        let lower = raw_a.min(raw_b);
        let upper = raw_a.max(raw_b);
        let eps = lower.abs().max(upper.abs()).max(1.0) * 1e-9;

        let low = Self::ceil_to_i128(lower - eps).clamp(raw_min, raw_max);
        let high = Self::floor_to_i128(upper + eps).clamp(raw_min, raw_max);

        if low <= high {
            (low, high)
        } else {
            (raw_min, raw_max)
        }
    }

    fn choose_test_raw(low: i128, high: i128, ordinal: usize, prefer_bounds: bool) -> i128 {
        fn push_candidate(candidates: &mut Vec<i128>, value: Option<i128>, low: i128, high: i128) {
            let Some(value) = value else {
                return;
            };

            if value < low || value > high || candidates.contains(&value) {
                return;
            }

            candidates.push(value);
        }

        let midpoint = low
            .checked_add(high)
            .map(|v| v / 2)
            .or_else(|| if low <= 0 && high >= 0 { Some(0) } else { None });

        let mut candidates = Vec::new();
        let ordered = if prefer_bounds {
            [
                Some(low),
                Some(high),
                midpoint,
                low.checked_add(1),
                high.checked_sub(1),
                Some(0),
                Some(1),
                Some(-1),
                Some(2),
                Some(-2),
            ]
        } else {
            [
                Some(0),
                Some(1),
                Some(-1),
                Some(2),
                Some(-2),
                midpoint,
                Some(low),
                Some(high),
                low.checked_add(1),
                high.checked_sub(1),
            ]
        };

        for candidate in ordered {
            push_candidate(&mut candidates, candidate, low, high);
        }

        if candidates.is_empty() {
            low
        } else {
            candidates[ordinal % candidates.len()]
        }
    }

    fn integer_raw_range(layout: &SignalLayout) -> (i128, i128) {
        match layout.value_type {
            ValueType::Signed => {
                if layout.size >= 128 {
                    (i128::MIN, i128::MAX)
                } else {
                    let magnitude = 1i128 << (layout.size - 1);
                    (-magnitude, magnitude - 1)
                }
            }
            ValueType::Unsigned => {
                if layout.size >= 127 {
                    (0, i128::MAX)
                } else {
                    (0, (1i128 << layout.size) - 1)
                }
            }
        }
    }

    fn ceil_to_i128(value: f64) -> i128 {
        if value <= i128::MIN as f64 {
            i128::MIN
        } else if value >= i128::MAX as f64 {
            i128::MAX
        } else {
            value.ceil() as i128
        }
    }

    fn floor_to_i128(value: f64) -> i128 {
        if value <= i128::MIN as f64 {
            i128::MIN
        } else if value >= i128::MAX as f64 {
            i128::MAX
        } else {
            value.floor() as i128
        }
    }

    fn test_unknown_frame_id(messages: &[Message]) -> MessageId {
        let mut standard_ids = BTreeSet::new();
        let mut extended_ids = BTreeSet::new();

        for msg in messages {
            match msg.id {
                MessageId::Standard(id) => {
                    standard_ids.insert(id);
                }
                MessageId::Extended(id) => {
                    extended_ids.insert(id);
                }
            }
        }

        for id in 0..=0x7ffu16 {
            if !standard_ids.contains(&id) {
                return MessageId::Standard(id);
            }
        }

        for id in 0..=0x1fff_ffffu32 {
            if !extended_ids.contains(&id) {
                return MessageId::Extended(id);
            }
        }

        MessageId::Extended(0x1fff_ffff)
    }

    fn format_can_id_expr(id: &MessageId) -> String {
        match id {
            MessageId::Standard(id) => format!("CanId::standard({})", id),
            MessageId::Extended(id) => format!("CanId::extended({})", id),
        }
    }

    fn invalid_enum_raw_value(signal: &Signal, file: &DbcFile) -> Option<i128> {
        let enum_idx = signal.signal_value_enum_idx?;
        let enum_def = &file.signal_value_enums[enum_idx.0];
        let used = enum_def
            .variants
            .iter()
            .map(|variant| variant.value as i128)
            .collect::<BTreeSet<_>>();
        let layout = &file.signal_layouts[signal.layout.0];
        let (low, high) = Self::integer_raw_range(layout);

        Self::choose_unused_raw_value(low, high, &used)
    }

    fn invalid_mux_raw_value(
        mux_signal: &Signal,
        muxed: &BTreeMap<u64, Vec<&Signal>>,
        file: &DbcFile,
    ) -> Option<i128> {
        let used = muxed.keys().map(|value| *value as i128).collect();
        let layout = &file.signal_layouts[mux_signal.layout.0];
        let (low, high) = Self::integer_raw_range(layout);

        Self::choose_unused_raw_value(low, high, &used)
    }

    fn choose_unused_raw_value(low: i128, high: i128, used: &BTreeSet<i128>) -> Option<i128> {
        if low > high {
            return None;
        }

        let mut candidates = Vec::new();
        let mut push_candidate = |value: Option<i128>| {
            let Some(value) = value else {
                return;
            };

            if value >= low
                && value <= high
                && !used.contains(&value)
                && !candidates.contains(&value)
            {
                candidates.push(value);
            }
        };

        for value in [
            Some(low),
            Some(high),
            Some(0),
            Some(1),
            Some(-1),
            Some(2),
            Some(-2),
        ] {
            push_candidate(value);
        }

        for value in used {
            push_candidate(value.checked_sub(1));
            push_candidate(value.checked_add(1));
        }

        if let Some(span) = high.checked_sub(low) {
            if span <= 512 {
                for value in low..=high {
                    push_candidate(Some(value));
                }
            } else {
                let limit = used.len().saturating_add(8).min(512);
                for offset in 0..=limit {
                    let offset = offset as i128;
                    push_candidate(low.checked_add(offset));
                    push_candidate(high.checked_sub(offset));
                    push_candidate(Some(offset));
                    if offset != 0 {
                        push_candidate(Some(-offset));
                    }
                }
            }
        }

        candidates.into_iter().next()
    }

    fn parse_can(out: &mut Generator, messages: &[Message], config: &CodegenConfig) {
        let variant_types = if messages.is_empty() {
            "std::monostate".to_string()
        } else {
            messages
                .iter()
                .map(|m| m.name.upper_camel())
                .collect::<Vec<_>>()
                .join(", ")
        };
        cpp_code_injections(out, config, CppCodeInjectionPoint::MessageVariant);
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
