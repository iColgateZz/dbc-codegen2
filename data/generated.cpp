#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
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
[[nodiscard]] constexpr T extract_le(const uint8_t* data, std::size_t start, std::size_t end) noexcept {
  using U = std::make_unsigned_t<T>;
  U result = 0;
  const std::size_t len = end - start;
  for (std::size_t i = 0; i < len; ++i) {
    const std::size_t bit_idx = start + i;
    result |= static_cast<U>((data[bit_idx / 8] >> (bit_idx % 8)) & 0x1u) << i;
  };
  if constexpr (std::is_signed_v<T>) {
    if (len == 0) return T(0);
    if (len < sizeof(U) * 8) {
      const U sign_bit = static_cast<U>(U(1) << (len - 1));
      if (result & sign_bit) {
        result |= static_cast<U>(~U(0) << len);
      };
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
    if (len == 0) return T(0);
    if (len < sizeof(U) * 8) {
      const U sign_bit = static_cast<U>(U(1) << (len - 1));
      if (result & sign_bit) {
        result |= static_cast<U>(~U(0) << len);
      };
    };
  };
  return static_cast<T>(result);
};

template <typename T>
constexpr void insert_le(uint8_t* data, std::size_t start, std::size_t end, T value) noexcept {
  using U = std::make_unsigned_t<T>;
  U v = static_cast<U>(value);
  const std::size_t len = end - start;
  for (std::size_t i = 0; i < len; ++i) {
    const std::size_t bit_idx = start + i;
    const uint8_t bit = static_cast<uint8_t>((v >> i) & 0x1u);
    data[bit_idx / 8] &= ~static_cast<uint8_t>(1u << (bit_idx % 8));
    data[bit_idx / 8] |= static_cast<uint8_t>(bit << (bit_idx % 8));
  };
};

template <typename T>
constexpr void insert_be(uint8_t* data, std::size_t start, std::size_t end, T value) noexcept {
  using U = std::make_unsigned_t<T>;
  U v = static_cast<U>(value);
  const std::size_t len = end - start;
  for (std::size_t i = 0; i < len; ++i) {
    const std::size_t bit_idx = start + i;
    const uint8_t bit = static_cast<uint8_t>((v >> (len - 1 - i)) & 0x1u);
    data[bit_idx / 8] &= ~static_cast<uint8_t>(1u << (7 - bit_idx % 8));
    data[bit_idx / 8] |= static_cast<uint8_t>(bit << (7 - bit_idx % 8));
  };
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
  
  [[nodiscard]] std::array<uint8_t, LEN> serialize() const noexcept {
    std::array<uint8_t, LEN> data{};
    detail::insert_le<uint8_t>(data.data(), 0, 8, static_cast<uint8_t>(driver_heartbeat_cmd));
    return data;
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
  
  [[nodiscard]] std::array<uint8_t, LEN> serialize() const noexcept {
    std::array<uint8_t, LEN> data{};
    detail::insert_le<uint8_t>(data.data(), 0, 8, static_cast<uint8_t>((io_debug_test_unsigned  - 0) / 1));
    detail::insert_le<uint8_t>(data.data(), 8, 16, static_cast<uint8_t>(io_debug_test_enum));
    detail::insert_le<int8_t>(data.data(), 16, 24, static_cast<int8_t>((io_debug_test_signed  - 0) / 1));
    detail::insert_le<uint8_t>(data.data(), 24, 32, static_cast<uint8_t>((io_debug_test_float - 0) / 0.5));
    return data;
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
  
  [[nodiscard]] std::array<uint8_t, LEN> serialize() const noexcept {
    std::array<uint8_t, LEN> data{};
    detail::insert_le<int8_t>(data.data(), 0, 4, static_cast<int8_t>((motor_cmd_steer  - -5) / 1));
    detail::insert_le<uint8_t>(data.data(), 4, 8, static_cast<uint8_t>((motor_cmd_drive  - 0) / 1));
    return data;
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
  
  [[nodiscard]] std::array<uint8_t, LEN> serialize() const noexcept {
    std::array<uint8_t, LEN> data{};
    detail::insert_le<uint8_t>(data.data(), 0, 1, static_cast<uint8_t>((motor_status_wheel_error  - 0) / 1));
    detail::insert_le<uint16_t>(data.data(), 8, 24, static_cast<uint16_t>((motor_status_speed_kph - 0) / 0.001));
    return data;
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
  
  void encode_into(uint8_t* data) const noexcept {
    detail::insert_le<uint16_t>(data, 16, 28, static_cast<uint16_t>((sensor_sonars_left - 0) / 0.1));
    detail::insert_le<uint16_t>(data, 28, 40, static_cast<uint16_t>((sensor_sonars_middle - 0) / 0.1));
    detail::insert_le<uint16_t>(data, 40, 52, static_cast<uint16_t>((sensor_sonars_right - 0) / 0.1));
    detail::insert_le<uint16_t>(data, 52, 64, static_cast<uint16_t>((sensor_sonars_rear - 0) / 0.1));
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
  
  void encode_into(uint8_t* data) const noexcept {
    detail::insert_le<uint16_t>(data, 16, 28, static_cast<uint16_t>((sensor_sonars_no_filt_left - 0) / 0.1));
    detail::insert_le<uint16_t>(data, 28, 40, static_cast<uint16_t>((sensor_sonars_no_filt_middle - 0) / 0.1));
    detail::insert_le<uint16_t>(data, 40, 52, static_cast<uint16_t>((sensor_sonars_no_filt_right - 0) / 0.1));
    detail::insert_le<uint16_t>(data, 52, 64, static_cast<uint16_t>((sensor_sonars_no_filt_rear - 0) / 0.1));
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
  
  [[nodiscard]] std::array<uint8_t, LEN> serialize() const noexcept {
    std::array<uint8_t, LEN> data{};
    detail::insert_le<uint16_t>(data.data(), 4, 16, static_cast<uint16_t>((sensor_sonars_err_count  - 0) / 1));
    std::visit([&data](const auto& inner) noexcept {
      if constexpr (std::is_same_v<std::decay_t<decltype(inner)>, SensorSonarsMux0>) {
        detail::insert_le<uint8_t>(data.data(), 0, 4, static_cast<uint8_t>(0));
        inner.encode_into(data.data());
      };
      if constexpr (std::is_same_v<std::decay_t<decltype(inner)>, SensorSonarsMux1>) {
        detail::insert_le<uint8_t>(data.data(), 0, 4, static_cast<uint8_t>(1));
        inner.encode_into(data.data());
      };
    }, mux);
    return data;
  };
  
};

using CanMsg = std::variant<DriverHeartbeat, IoDebug, MotorCmd, MotorStatus, SensorSonars>;

[[nodiscard]]
inline std::expected<CanMsg, CanError>
parse_can(uint32_t id, std::span<const uint8_t> data) noexcept {
  switch (id) {
    case DriverHeartbeat::ID:
     {
      if (data.size() < DriverHeartbeat::LEN) return std::unexpected(CanError::InvalidLength);
      auto r = DriverHeartbeat::parse(std::span<const uint8_t, DriverHeartbeat::LEN>(data.data(), DriverHeartbeat::LEN));
      if (!r) return std::unexpected(r.error());
      return CanMsg{std::move(*r)};
    };
    case IoDebug::ID:
     {
      if (data.size() < IoDebug::LEN) return std::unexpected(CanError::InvalidLength);
      auto r = IoDebug::parse(std::span<const uint8_t, IoDebug::LEN>(data.data(), IoDebug::LEN));
      if (!r) return std::unexpected(r.error());
      return CanMsg{std::move(*r)};
    };
    case MotorCmd::ID:
     {
      if (data.size() < MotorCmd::LEN) return std::unexpected(CanError::InvalidLength);
      auto r = MotorCmd::parse(std::span<const uint8_t, MotorCmd::LEN>(data.data(), MotorCmd::LEN));
      if (!r) return std::unexpected(r.error());
      return CanMsg{std::move(*r)};
    };
    case MotorStatus::ID:
     {
      if (data.size() < MotorStatus::LEN) return std::unexpected(CanError::InvalidLength);
      auto r = MotorStatus::parse(std::span<const uint8_t, MotorStatus::LEN>(data.data(), MotorStatus::LEN));
      if (!r) return std::unexpected(r.error());
      return CanMsg{std::move(*r)};
    };
    case SensorSonars::ID:
     {
      if (data.size() < SensorSonars::LEN) return std::unexpected(CanError::InvalidLength);
      auto r = SensorSonars::parse(std::span<const uint8_t, SensorSonars::LEN>(data.data(), SensorSonars::LEN));
      if (!r) return std::unexpected(r.error());
      return CanMsg{std::move(*r)};
    };
    default: return std::unexpected(CanError::UnknownId);
  };
};
