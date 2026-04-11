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

template <typename T>
[[nodiscard]] constexpr T extract_le(const uint8_t* data, std::size_t start, std::size_t end) noexcept {
  using U = std::make_unsigned_t<T>;
  U result = 0;
  const std::size_t len = end - start;
  for (std::size_t i = 0; i < len; ++i) {
    const std::size_t bit_idx = start + i;
    result |= static_cast<U>((data[bit_idx / 8] >> (bit_idx % 8)) & 0x1u) << i;
  };
  if constexpr (std::is_signed_v<T>) {
    if (len < sizeof(U) * 8 && (result & (U(1) << (len - 1)))) {
      result |= ~U(0) << len;
    };
  };
  return static_cast<T>(result);
};

template <typename T>
[[nodiscard]] constexpr T extract_be(const uint8_t* data, std::size_t start, std::size_t end) noexcept {
  using U = std::make_unsigned_t<T>;
  U result = 0;
  const std::size_t len = end - start;
  for (std::size_t i = 0; i < len; ++i) {
    const std::size_t bit_idx = start + i;
    result = (result << 1) | static_cast<U>((data[bit_idx / 8] >> (7 - bit_idx % 8)) & 0x1u);
  };
  if constexpr (std::is_signed_v<T>) {
    if (len < sizeof(U) * 8 && (result & (U(1) << (len - 1)))) {
      result |= ~U(0) << len;
    };
  };
  return static_cast<T>(result);
};

} // namespace detail

enum class DriverHeartbeatCmd : uint8_t {
  Reboot = 2,
  Sync = 1,
  Noop = 0,
};

[[nodiscard]] constexpr std::expected<DriverHeartbeatCmd, CanError>
driver_heartbeat_cmd_from_raw(uint8_t v) noexcept {
  switch (v) {
    case 2: return DriverHeartbeatCmd::Reboot;
    case 1: return DriverHeartbeatCmd::Sync;
    case 0: return DriverHeartbeatCmd::Noop;
    default: return std::unexpected(CanError::InvalidData);
  };
};

struct DriverHeartbeat {
  static constexpr uint16_t ID = 100;
  static constexpr std::size_t LEN = 1;
  
  DriverHeartbeatCmd driver_heartbeat_cmd;
  
  [[nodiscard]] static std::expected<DriverHeartbeat, CanError>
  parse(std::span<const uint8_t, LEN> data) noexcept {
    const uint8_t raw_driver_heartbeat_cmd = detail::extract_le<uint8_t>(data.data(), 0, 8);
    auto driver_heartbeat_cmd_exp = driver_heartbeat_cmd_from_raw(raw_driver_heartbeat_cmd);
    if (!driver_heartbeat_cmd_exp) return std::unexpected(driver_heartbeat_cmd_exp.error());
    return DriverHeartbeat{ .driver_heartbeat_cmd = *driver_heartbeat_cmd_exp };
  };
  
};

enum class IoDebugTestEnum : uint8_t {
  Two = 2,
  One = 1,
};

[[nodiscard]] constexpr std::expected<IoDebugTestEnum, CanError>
io_debug_test_enum_from_raw(uint8_t v) noexcept {
  switch (v) {
    case 2: return IoDebugTestEnum::Two;
    case 1: return IoDebugTestEnum::One;
    default: return std::unexpected(CanError::InvalidData);
  };
};

struct IoDebug {
  static constexpr uint16_t ID = 500;
  static constexpr std::size_t LEN = 4;
  
  uint8_t io_debug_test_unsigned;
  IoDebugTestEnum io_debug_test_enum;
  int8_t io_debug_test_signed;
  double io_debug_test_float;
  
  [[nodiscard]] static std::expected<IoDebug, CanError>
  parse(std::span<const uint8_t, LEN> data) noexcept {
    const uint8_t raw_io_debug_test_unsigned = detail::extract_le<uint8_t>(data.data(), 0, 8);
    const uint8_t io_debug_test_unsigned = static_cast<uint8_t>(raw_io_debug_test_unsigned) * 1 + 0;
    const uint8_t raw_io_debug_test_enum = detail::extract_le<uint8_t>(data.data(), 8, 16);
    auto io_debug_test_enum_exp = io_debug_test_enum_from_raw(raw_io_debug_test_enum);
    if (!io_debug_test_enum_exp) return std::unexpected(io_debug_test_enum_exp.error());
    const int8_t raw_io_debug_test_signed = detail::extract_le<int8_t>(data.data(), 16, 24);
    const int8_t io_debug_test_signed = static_cast<int8_t>(raw_io_debug_test_signed) * 1 + 0;
    const uint8_t raw_io_debug_test_float = detail::extract_le<uint8_t>(data.data(), 24, 32);
    const double io_debug_test_float = static_cast<double>(raw_io_debug_test_float) * 0.5 + 0;
    return IoDebug{ .io_debug_test_unsigned = io_debug_test_unsigned, .io_debug_test_enum = *io_debug_test_enum_exp, .io_debug_test_signed = io_debug_test_signed, .io_debug_test_float = io_debug_test_float };
  };
  
};

struct MotorCmd {
  static constexpr uint16_t ID = 101;
  static constexpr std::size_t LEN = 1;
  
  int8_t motor_cmd_steer;
  uint8_t motor_cmd_drive;
  
  [[nodiscard]] static std::expected<MotorCmd, CanError>
  parse(std::span<const uint8_t, LEN> data) noexcept {
    const int8_t raw_motor_cmd_steer = detail::extract_le<int8_t>(data.data(), 0, 4);
    const int8_t motor_cmd_steer = static_cast<int8_t>(raw_motor_cmd_steer) * 1 + -5;
    const uint8_t raw_motor_cmd_drive = detail::extract_le<uint8_t>(data.data(), 4, 8);
    const uint8_t motor_cmd_drive = static_cast<uint8_t>(raw_motor_cmd_drive) * 1 + 0;
    return MotorCmd{ .motor_cmd_steer = motor_cmd_steer, .motor_cmd_drive = motor_cmd_drive };
  };
  
};

struct MotorStatus {
  static constexpr uint16_t ID = 400;
  static constexpr std::size_t LEN = 3;
  
  uint8_t motor_status_wheel_error;
  double motor_status_speed_kph;
  
  [[nodiscard]] static std::expected<MotorStatus, CanError>
  parse(std::span<const uint8_t, LEN> data) noexcept {
    const uint8_t raw_motor_status_wheel_error = detail::extract_le<uint8_t>(data.data(), 0, 1);
    const uint8_t motor_status_wheel_error = static_cast<uint8_t>(raw_motor_status_wheel_error) * 1 + 0;
    const uint16_t raw_motor_status_speed_kph = detail::extract_le<uint16_t>(data.data(), 8, 24);
    const double motor_status_speed_kph = static_cast<double>(raw_motor_status_speed_kph) * 0.001 + 0;
    return MotorStatus{ .motor_status_wheel_error = motor_status_wheel_error, .motor_status_speed_kph = motor_status_speed_kph };
  };
  
};

struct SensorSonarsMux0 {
  double sensor_sonars_left;
  double sensor_sonars_middle;
  double sensor_sonars_right;
  double sensor_sonars_rear;
  
  [[nodiscard]] static std::expected<SensorSonarsMux0, CanError>
  decode_from(const uint8_t* data) noexcept {
    const uint16_t raw_sensor_sonars_left = detail::extract_le<uint16_t>(data, 16, 28);
    const double sensor_sonars_left = static_cast<double>(raw_sensor_sonars_left) * 0.1 + 0;
    const uint16_t raw_sensor_sonars_middle = detail::extract_le<uint16_t>(data, 28, 40);
    const double sensor_sonars_middle = static_cast<double>(raw_sensor_sonars_middle) * 0.1 + 0;
    const uint16_t raw_sensor_sonars_right = detail::extract_le<uint16_t>(data, 40, 52);
    const double sensor_sonars_right = static_cast<double>(raw_sensor_sonars_right) * 0.1 + 0;
    const uint16_t raw_sensor_sonars_rear = detail::extract_le<uint16_t>(data, 52, 64);
    const double sensor_sonars_rear = static_cast<double>(raw_sensor_sonars_rear) * 0.1 + 0;
    return SensorSonarsMux0{ .sensor_sonars_left = sensor_sonars_left, .sensor_sonars_middle = sensor_sonars_middle, .sensor_sonars_right = sensor_sonars_right, .sensor_sonars_rear = sensor_sonars_rear };
  };
  
};

struct SensorSonarsMux1 {
  double sensor_sonars_no_filt_left;
  double sensor_sonars_no_filt_middle;
  double sensor_sonars_no_filt_right;
  double sensor_sonars_no_filt_rear;
  
  [[nodiscard]] static std::expected<SensorSonarsMux1, CanError>
  decode_from(const uint8_t* data) noexcept {
    const uint16_t raw_sensor_sonars_no_filt_left = detail::extract_le<uint16_t>(data, 16, 28);
    const double sensor_sonars_no_filt_left = static_cast<double>(raw_sensor_sonars_no_filt_left) * 0.1 + 0;
    const uint16_t raw_sensor_sonars_no_filt_middle = detail::extract_le<uint16_t>(data, 28, 40);
    const double sensor_sonars_no_filt_middle = static_cast<double>(raw_sensor_sonars_no_filt_middle) * 0.1 + 0;
    const uint16_t raw_sensor_sonars_no_filt_right = detail::extract_le<uint16_t>(data, 40, 52);
    const double sensor_sonars_no_filt_right = static_cast<double>(raw_sensor_sonars_no_filt_right) * 0.1 + 0;
    const uint16_t raw_sensor_sonars_no_filt_rear = detail::extract_le<uint16_t>(data, 52, 64);
    const double sensor_sonars_no_filt_rear = static_cast<double>(raw_sensor_sonars_no_filt_rear) * 0.1 + 0;
    return SensorSonarsMux1{ .sensor_sonars_no_filt_left = sensor_sonars_no_filt_left, .sensor_sonars_no_filt_middle = sensor_sonars_no_filt_middle, .sensor_sonars_no_filt_right = sensor_sonars_no_filt_right, .sensor_sonars_no_filt_rear = sensor_sonars_no_filt_rear };
  };
  
};

using SensorSonarsMux = std::variant<SensorSonarsMux0, SensorSonarsMux1>;

struct SensorSonars {
  static constexpr uint16_t ID = 200;
  static constexpr std::size_t LEN = 8;
  
  uint16_t sensor_sonars_err_count;
  SensorSonarsMux mux;
  
  [[nodiscard]] static std::expected<SensorSonars, CanError>
  parse(std::span<const uint8_t, LEN> data) noexcept {
    const uint16_t raw_sensor_sonars_err_count = detail::extract_le<uint16_t>(data.data(), 4, 16);
    const uint16_t sensor_sonars_err_count = static_cast<uint16_t>(raw_sensor_sonars_err_count) * 1 + 0;
    const uint8_t mux_raw = detail::extract_le<uint8_t>(data.data(), 0, 4);
    switch (mux_raw) {
      case 0:
       {
        auto inner = SensorSonarsMux0::decode_from(data.data());
        if (!inner) return std::unexpected(inner.error());
        return SensorSonars{ .sensor_sonars_err_count = sensor_sonars_err_count, .mux = *inner };
      };
      case 1:
       {
        auto inner = SensorSonarsMux1::decode_from(data.data());
        if (!inner) return std::unexpected(inner.error());
        return SensorSonars{ .sensor_sonars_err_count = sensor_sonars_err_count, .mux = *inner };
      };
      default: return std::unexpected(CanError::InvalidData);
    };
  };
  
};

