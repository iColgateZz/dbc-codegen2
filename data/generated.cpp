#pragma once

#include <array>
#include <bit>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <expected>
#include <span>
#include <variant>

enum class CanError : uint8_t {
  UnknownId,
  InvalidLength,
  InvalidData,
};

namespace detail {

template <typename T>
[[nodiscard]] constexpr T read_le(const uint8_t *d) noexcept {
  T v{};
  std::memcpy(&v, d, sizeof(T));
  if constexpr (std::endian::native == std::endian::big) v = std::byteswap(v);
  return v;
};

template <typename T>
[[nodiscard]] constexpr T read_be(const uint8_t *d) noexcept {
  T v{};
  std::memcpy(&v, d, sizeof(T));
  if constexpr (std::endian::native == std::endian::little) v = std::byteswap(v);
  return v;
};

template <typename T>
constexpr void write_le(uint8_t *d, T v) noexcept {
  if constexpr (std::endian::native == std::endian::big) v = std::byteswap(v);
  std::memcpy(d, &v, sizeof(T));
};

template <typename T>
constexpr void write_be(uint8_t *d, T v) noexcept {
  if constexpr (std::endian::native == std::endian::little) v = std::byteswap(v);
  std::memcpy(d, &v, sizeof(T));
};

} // namespace detail

struct DRIVER_HEARTBEAT {
};

struct IO_DEBUG {
};

struct MOTOR_CMD {
};

struct MOTOR_STATUS {
};

struct SENSOR_SONARS {
};

