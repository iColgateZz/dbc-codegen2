// Requires GCC >= 13 for std::expected (fully supported in GCC 14 /
// Ubuntu 24.04 default)

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
  if constexpr (std::endian::native == std::endian::big)
    v = std::byteswap(v);
  return v;
}

template <typename T>
[[nodiscard]] constexpr T read_be(const uint8_t *d) noexcept {
  T v{};
  std::memcpy(&v, d, sizeof(T));
  if constexpr (std::endian::native == std::endian::little)
    v = std::byteswap(v);
  return v;
}

template <typename T> constexpr void write_le(uint8_t *d, T v) noexcept {
  if constexpr (std::endian::native == std::endian::big)
    v = std::byteswap(v);
  std::memcpy(d, &v, sizeof(T));
}

template <typename T> constexpr void write_be(uint8_t *d, T v) noexcept {
  if constexpr (std::endian::native == std::endian::little)
    v = std::byteswap(v);
  std::memcpy(d, &v, sizeof(T));
}

} // namespace detail

enum class EngineMode : uint8_t {
  Off = 0,
  Idle = 1,
  Drive = 2,
  Sport = 3,
};

struct EngineData {
  static constexpr uint32_t ID = 100u;
  static constexpr std::size_t LEN = 8u;

  float rpm;
  float speed;
  EngineMode engine_mode;

  [[nodiscard]]
  static std::expected<EngineData, CanError>
  parse(uint32_t id, std::span<const uint8_t, LEN> data) noexcept {
    if (id != ID)
      return std::unexpected(CanError::UnknownId);

    const uint16_t raw_rpm = detail::read_le<uint16_t>(&data[0]);
    const uint16_t raw_speed = detail::read_le<uint16_t>(&data[2]);

    return EngineData{
        .rpm = raw_rpm * 0.125f,
        .speed = raw_speed * 0.01f,
        .engine_mode = static_cast<EngineMode>(data[4]),
    };
  }

  [[nodiscard]]
  std::pair<uint32_t, std::array<uint8_t, LEN>> serialize() const noexcept {
    std::array<uint8_t, LEN> buf{};

    detail::write_le<uint16_t>(&buf[0], static_cast<uint16_t>(rpm / 0.125f));
    detail::write_le<uint16_t>(&buf[2], static_cast<uint16_t>(speed / 0.01f));
    buf[4] = static_cast<uint8_t>(engine_mode);

    return {ID, buf};
  }
};

struct OtherData {
  static constexpr uint32_t ID = 101u;
  static constexpr std::size_t LEN = 8u;

  float something;

  [[nodiscard]]
  static std::expected<OtherData, CanError>
  parse(uint32_t id, std::span<const uint8_t, LEN> data) noexcept {
    if (id != ID)
      return std::unexpected(CanError::UnknownId);

    const uint32_t raw = detail::read_be<uint32_t>(&data[0]);
    return OtherData{.something = static_cast<float>(raw) * 0.001f};
  }

  [[nodiscard]]
  std::pair<uint32_t, std::array<uint8_t, LEN>> serialize() const noexcept {
    std::array<uint8_t, LEN> buf{};

    detail::write_be<uint32_t>(&buf[0], static_cast<uint32_t>(something / 0.001f));

    return {ID, buf};
  }
};

using CanMsg = std::variant<EngineData, OtherData>;

[[nodiscard]]
inline std::expected<CanMsg, CanError>
parse_can(uint32_t id, std::span<const uint8_t, 8> data) noexcept {
  switch (id) {
  case EngineData::ID: {
    auto r = EngineData::parse(id, data);
    if (!r)
      return std::unexpected(r.error());
    return CanMsg{std::move(*r)};
  }
  case OtherData::ID: {
    auto r = OtherData::parse(id, data);
    if (!r)
      return std::unexpected(r.error());
    return CanMsg{std::move(*r)};
  }
  default:
    return std::unexpected(CanError::UnknownId);
  }
}
