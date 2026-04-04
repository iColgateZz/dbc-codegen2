use crate::{DbcFile, codegen::Generator, empty, end_block, line, start_block};

pub struct CppGen;

impl CppGen {
    pub fn generate(file: &DbcFile) -> String {
        let mut out = Generator::new();

        line!(out, "#pragma once");
        empty!(out);

        Self::includes(&mut out);
        Self::errors(&mut out);
        Self::endian_read_and_write(&mut out);
        Self::messages(&mut out, file);

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
        start_block!(out, "[[nodiscard]] constexpr T {name}(const uint8_t *d) noexcept");
        line!(out, "T v{{}};");
        line!(out, "std::memcpy(&v, d, sizeof(T));");
        line!(out, "if constexpr (std::endian::native == std::endian::{swap_if}) v = std::byteswap(v);");
        end_block!(out, "return v;");
        empty!(out);
    }
    
    fn endian_write_fn(out: &mut Generator, name: &str, swap_if: &str) {
        line!(out, "template <typename T>");
        start_block!(out, "constexpr void {name}(uint8_t *d, T v) noexcept");
        line!(out, "if constexpr (std::endian::native == std::endian::{swap_if}) v = std::byteswap(v);");
        end_block!(out, "std::memcpy(d, &v, sizeof(T));");
        empty!(out);
    }
    
    fn endian_read_and_write(out: &mut Generator) {
        line!(out, "namespace detail {{");
        empty!(out);
    
        Self::endian_read_fn(out, "read_le", "big");
        Self::endian_read_fn(out, "read_be", "little");
        Self::endian_write_fn(out, "write_le", "big");
        Self::endian_write_fn(out, "write_be", "little");
    
        line!(out, "}} // namespace detail");
        empty!(out);
    }

    fn messages(out: &mut Generator, file: &DbcFile) {
        for message in &file.messages {
            start_block!(out, "struct {}", message.name.0);
            end_block!(out, "");
            empty!(out);
        }
    }
}
