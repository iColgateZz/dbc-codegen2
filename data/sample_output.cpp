// Requires GCC >= 13 for std::expected (fully supported in GCC 14 /
// Ubuntu 24.04 default)

#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <expected>
#include <span>
#include <variant>

enum class CanError : uint8_t {
  UnknownId,
  InvalidLength,
  InvalidData,
};

namespace detail {

[[nodiscard]] constexpr uint16_t read_u16_le(const uint8_t *d) noexcept {
  return static_cast<uint16_t>(d[0]) |
         static_cast<uint16_t>(static_cast<uint16_t>(d[1]) << 8u);
}

[[nodiscard]] constexpr uint32_t read_u32_le(const uint8_t *d) noexcept {
  return static_cast<uint32_t>(d[0]) | (static_cast<uint32_t>(d[1]) << 8u) |
         (static_cast<uint32_t>(d[2]) << 16u) |
         (static_cast<uint32_t>(d[3]) << 24u);
}

constexpr void write_u16_le(uint8_t *d, uint16_t v) noexcept {
  d[0] = static_cast<uint8_t>(v);
  d[1] = static_cast<uint8_t>(v >> 8u);
}

constexpr void write_u32_le(uint8_t *d, uint32_t v) noexcept {
  d[0] = static_cast<uint8_t>(v);
  d[1] = static_cast<uint8_t>(v >> 8u);
  d[2] = static_cast<uint8_t>(v >> 16u);
  d[3] = static_cast<uint8_t>(v >> 24u);
}
} // namespace detail

enum class EngineMode : uint8_t {
  Off = 0,
  Idle = 1,
  Drive = 2,
  Sport = 3,
};

struct EngineModeValue {
  EngineMode mode;
  uint8_t raw;

  static constexpr EngineModeValue from_raw(uint8_t v) noexcept {
    switch (v) {
    case 0:
      return {EngineMode::Off, 0};
    case 1:
      return {EngineMode::Idle, 1};
    case 2:
      return {EngineMode::Drive, 2};
    case 3:
      return {EngineMode::Sport, 3};
    default:
      return {static_cast<EngineMode>(v), v};
    }
  }

  [[nodiscard]] constexpr uint8_t to_raw() const noexcept { return raw; }

  void debug_print() const noexcept {
    switch (mode) {
    case EngineMode::Off:
      std::printf("Off");
      break;
    case EngineMode::Idle:
      std::printf("Idle");
      break;
    case EngineMode::Drive:
      std::printf("Drive");
      break;
    case EngineMode::Sport:
      std::printf("Sport");
      break;
    default:
      std::printf("_Other(%u)", raw);
      break;
    }
  }
};

struct EngineData {
  static constexpr uint32_t ID = 100u;
  static constexpr std::size_t LEN = 8u;

  float rpm;
  float speed;
  EngineModeValue engine_mode;

  [[nodiscard]]
  static std::expected<EngineData, CanError>
  parse(uint32_t id, std::span<const uint8_t, LEN> data) noexcept {
    if (id != ID)
      return std::unexpected(CanError::UnknownId);

    const uint16_t raw_rpm = detail::read_u16_le(&data[0]);
    const uint16_t raw_speed = detail::read_u16_le(&data[2]);

    return EngineData{
        .rpm = raw_rpm * 0.125f,
        .speed = raw_speed * 0.01f,
        .engine_mode = EngineModeValue::from_raw(data[4]),
    };
  }

  [[nodiscard]]
  std::pair<uint32_t, std::array<uint8_t, LEN>> serialize() const noexcept {
    std::array<uint8_t, LEN> buf{};

    detail::write_u16_le(&buf[0], static_cast<uint16_t>(rpm / 0.125f));
    detail::write_u16_le(&buf[2], static_cast<uint16_t>(speed / 0.01f));
    buf[4] = engine_mode.to_raw();

    return {ID, buf};
  }

#ifndef CAN_NO_DEBUG
  void debug_print() const noexcept {
    std::printf("EngineData { rpm: %.3f, speed: %.3f, engine_mode: ",
                static_cast<double>(rpm), static_cast<double>(speed));
    engine_mode.debug_print();
    std::printf(" }\n");
  }
#endif
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

    const uint32_t raw = detail::read_u32_le(&data[0]);
    return OtherData{.something = static_cast<float>(raw) * 0.001f};
  }

  [[nodiscard]]
  std::pair<uint32_t, std::array<uint8_t, LEN>> serialize() const noexcept {
    std::array<uint8_t, LEN> buf{};

    detail::write_u32_le(&buf[0], static_cast<uint32_t>(something / 0.001f));

    return {ID, buf};
  }

#ifndef CAN_NO_DEBUG
  void debug_print() const noexcept {
    std::printf("OtherData { something: %.3f }\n",
                static_cast<double>(something));
  }
#endif
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

#ifndef CAN_NO_DEBUG
inline void debug_print_can_msg(const CanMsg &msg) noexcept {
  std::visit([](const auto &m) { m.debug_print(); }, msg);
}
#endif
