#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <span>
#include <variant>
#include <utility>

enum class CanError : uint8_t {
  UnknownFrameId,
  UnknownMuxValue,
  InvalidPayloadSize,
  ValueOutOfRange,
  InvalidEnumValue,
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
    default: return static_cast<DriverHeartbeatCmd>(v);
  };
};

class DriverHeartbeat {
  public:
  static constexpr uint16_t ID = 100;
  static constexpr std::size_t LEN = 1;
  
  [[nodiscard]] static std::expected<DriverHeartbeat, CanError> create(
            DriverHeartbeatCmd driver_heartbeat_cmd
        ) noexcept {
    DriverHeartbeat msg{};
    if (auto r = msg.set_driver_heartbeat_cmd(driver_heartbeat_cmd); !r) return std::unexpected(r.error());
    return msg;
  };
  
  [[nodiscard]] static std::expected<DriverHeartbeat, CanError> try_from_frame(std::span<const uint8_t> frame) noexcept {
    if (frame.size() < LEN) return std::unexpected(CanError::InvalidPayloadSize);
    DriverHeartbeat msg{};
    std::memcpy(msg.data_.data(), frame.data(), LEN);
    return msg;
  };
  
  [[nodiscard]] std::expected<DriverHeartbeatCmd, CanError> driver_heartbeat_cmd() const noexcept {
    const uint8_t raw_driver_heartbeat_cmd = detail::extract_le<uint8_t>(data_.data(), 0, 8);
    return driver_heartbeat_cmd_from_raw(raw_driver_heartbeat_cmd);
  };
  
  std::expected<void, CanError> set_driver_heartbeat_cmd(DriverHeartbeatCmd driver_heartbeat_cmd) noexcept {
    detail::insert_le<uint8_t>(data_.data(), 0, 8, static_cast<uint8_t>(driver_heartbeat_cmd));
    return {};
  };
  
  [[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept { return data_; }
  
  private:
  std::array<uint8_t, LEN> data_{};
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
    default: return static_cast<IoDebugTestEnum>(v);
  };
};

class IoDebug {
  public:
  static constexpr uint16_t ID = 500;
  static constexpr std::size_t LEN = 4;
  
  [[nodiscard]] static std::expected<IoDebug, CanError> create(
            uint8_t io_debug_test_unsigned,
            IoDebugTestEnum io_debug_test_enum,
            int8_t io_debug_test_signed,
            double io_debug_test_float
        ) noexcept {
    IoDebug msg{};
    if (auto r = msg.set_io_debug_test_unsigned(io_debug_test_unsigned); !r) return std::unexpected(r.error());
    if (auto r = msg.set_io_debug_test_enum(io_debug_test_enum); !r) return std::unexpected(r.error());
    if (auto r = msg.set_io_debug_test_signed(io_debug_test_signed); !r) return std::unexpected(r.error());
    if (auto r = msg.set_io_debug_test_float(io_debug_test_float); !r) return std::unexpected(r.error());
    return msg;
  };
  
  [[nodiscard]] static std::expected<IoDebug, CanError> try_from_frame(std::span<const uint8_t> frame) noexcept {
    if (frame.size() < LEN) return std::unexpected(CanError::InvalidPayloadSize);
    IoDebug msg{};
    std::memcpy(msg.data_.data(), frame.data(), LEN);
    return msg;
  };
  
  [[nodiscard]] uint8_t io_debug_test_unsigned() const noexcept {
    const uint8_t raw_io_debug_test_unsigned = detail::extract_le<uint8_t>(data_.data(), 0, 8);
    return static_cast<uint8_t>(raw_io_debug_test_unsigned) * 1 + 0;
  };
  
  [[nodiscard]] std::expected<IoDebugTestEnum, CanError> io_debug_test_enum() const noexcept {
    const uint8_t raw_io_debug_test_enum = detail::extract_le<uint8_t>(data_.data(), 8, 16);
    return io_debug_test_enum_from_raw(raw_io_debug_test_enum);
  };
  
  [[nodiscard]] int8_t io_debug_test_signed() const noexcept {
    const int8_t raw_io_debug_test_signed = detail::extract_le<int8_t>(data_.data(), 16, 24);
    return static_cast<int8_t>(raw_io_debug_test_signed) * 1 + 0;
  };
  
  [[nodiscard]] double io_debug_test_float() const noexcept {
    const uint8_t raw_io_debug_test_float = detail::extract_le<uint8_t>(data_.data(), 24, 32);
    return static_cast<double>(raw_io_debug_test_float) * 0.5 + 0;
  };
  
  std::expected<void, CanError> set_io_debug_test_unsigned(uint8_t io_debug_test_unsigned) noexcept {
    if (io_debug_test_unsigned < 0 || io_debug_test_unsigned > 0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint8_t>(data_.data(), 0, 8, static_cast<uint8_t>((io_debug_test_unsigned - 0) / 1));
    return {};
  };
  
  std::expected<void, CanError> set_io_debug_test_enum(IoDebugTestEnum io_debug_test_enum) noexcept {
    detail::insert_le<uint8_t>(data_.data(), 8, 16, static_cast<uint8_t>(io_debug_test_enum));
    return {};
  };
  
  std::expected<void, CanError> set_io_debug_test_signed(int8_t io_debug_test_signed) noexcept {
    if (io_debug_test_signed < 0 || io_debug_test_signed > 0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<int8_t>(data_.data(), 16, 24, static_cast<int8_t>((io_debug_test_signed - 0) / 1));
    return {};
  };
  
  std::expected<void, CanError> set_io_debug_test_float(double io_debug_test_float) noexcept {
    if (io_debug_test_float < 0f || io_debug_test_float > 0f) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint8_t>(data_.data(), 24, 32, static_cast<uint8_t>((io_debug_test_float - 0) / 0.5));
    return {};
  };
  
  [[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept { return data_; }
  
  private:
  std::array<uint8_t, LEN> data_{};
};

class MotorCmd {
  public:
  static constexpr uint16_t ID = 101;
  static constexpr std::size_t LEN = 1;
  
  [[nodiscard]] static std::expected<MotorCmd, CanError> create(
            int8_t motor_cmd_steer,
            uint8_t motor_cmd_drive
        ) noexcept {
    MotorCmd msg{};
    if (auto r = msg.set_motor_cmd_steer(motor_cmd_steer); !r) return std::unexpected(r.error());
    if (auto r = msg.set_motor_cmd_drive(motor_cmd_drive); !r) return std::unexpected(r.error());
    return msg;
  };
  
  [[nodiscard]] static std::expected<MotorCmd, CanError> try_from_frame(std::span<const uint8_t> frame) noexcept {
    if (frame.size() < LEN) return std::unexpected(CanError::InvalidPayloadSize);
    MotorCmd msg{};
    std::memcpy(msg.data_.data(), frame.data(), LEN);
    return msg;
  };
  
  [[nodiscard]] int8_t motor_cmd_steer() const noexcept {
    const int8_t raw_motor_cmd_steer = detail::extract_le<int8_t>(data_.data(), 0, 4);
    return static_cast<int8_t>(raw_motor_cmd_steer) * 1 + -5;
  };
  
  [[nodiscard]] uint8_t motor_cmd_drive() const noexcept {
    const uint8_t raw_motor_cmd_drive = detail::extract_le<uint8_t>(data_.data(), 4, 8);
    return static_cast<uint8_t>(raw_motor_cmd_drive) * 1 + 0;
  };
  
  std::expected<void, CanError> set_motor_cmd_steer(int8_t motor_cmd_steer) noexcept {
    if (motor_cmd_steer < -5 || motor_cmd_steer > 5) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<int8_t>(data_.data(), 0, 4, static_cast<int8_t>((motor_cmd_steer - -5) / 1));
    return {};
  };
  
  std::expected<void, CanError> set_motor_cmd_drive(uint8_t motor_cmd_drive) noexcept {
    if (motor_cmd_drive < 0 || motor_cmd_drive > 9) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint8_t>(data_.data(), 4, 8, static_cast<uint8_t>((motor_cmd_drive - 0) / 1));
    return {};
  };
  
  [[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept { return data_; }
  
  private:
  std::array<uint8_t, LEN> data_{};
};

class MotorStatus {
  public:
  static constexpr uint16_t ID = 400;
  static constexpr std::size_t LEN = 3;
  
  [[nodiscard]] static std::expected<MotorStatus, CanError> create(
            uint8_t motor_status_wheel_error,
            double motor_status_speed_kph
        ) noexcept {
    MotorStatus msg{};
    if (auto r = msg.set_motor_status_wheel_error(motor_status_wheel_error); !r) return std::unexpected(r.error());
    if (auto r = msg.set_motor_status_speed_kph(motor_status_speed_kph); !r) return std::unexpected(r.error());
    return msg;
  };
  
  [[nodiscard]] static std::expected<MotorStatus, CanError> try_from_frame(std::span<const uint8_t> frame) noexcept {
    if (frame.size() < LEN) return std::unexpected(CanError::InvalidPayloadSize);
    MotorStatus msg{};
    std::memcpy(msg.data_.data(), frame.data(), LEN);
    return msg;
  };
  
  [[nodiscard]] uint8_t motor_status_wheel_error() const noexcept {
    const uint8_t raw_motor_status_wheel_error = detail::extract_le<uint8_t>(data_.data(), 0, 1);
    return static_cast<uint8_t>(raw_motor_status_wheel_error) * 1 + 0;
  };
  
  [[nodiscard]] double motor_status_speed_kph() const noexcept {
    const uint16_t raw_motor_status_speed_kph = detail::extract_le<uint16_t>(data_.data(), 8, 24);
    return static_cast<double>(raw_motor_status_speed_kph) * 0.001 + 0;
  };
  
  std::expected<void, CanError> set_motor_status_wheel_error(uint8_t motor_status_wheel_error) noexcept {
    if (motor_status_wheel_error < 0 || motor_status_wheel_error > 0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint8_t>(data_.data(), 0, 1, static_cast<uint8_t>((motor_status_wheel_error - 0) / 1));
    return {};
  };
  
  std::expected<void, CanError> set_motor_status_speed_kph(double motor_status_speed_kph) noexcept {
    if (motor_status_speed_kph < 0f || motor_status_speed_kph > 0f) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 8, 24, static_cast<uint16_t>((motor_status_speed_kph - 0) / 0.001));
    return {};
  };
  
  [[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept { return data_; }
  
  private:
  std::array<uint8_t, LEN> data_{};
};

class SensorSonarsMux0 {
  public:
  static constexpr std::size_t LEN = 8;
  
  [[nodiscard]] static std::expected<SensorSonarsMux0, CanError> create(
            double sensor_sonars_left,
            double sensor_sonars_middle,
            double sensor_sonars_right,
            double sensor_sonars_rear
        ) noexcept {
    SensorSonarsMux0 msg{};
    if (auto r = msg.set_sensor_sonars_left(sensor_sonars_left); !r) return std::unexpected(r.error());
    if (auto r = msg.set_sensor_sonars_middle(sensor_sonars_middle); !r) return std::unexpected(r.error());
    if (auto r = msg.set_sensor_sonars_right(sensor_sonars_right); !r) return std::unexpected(r.error());
    if (auto r = msg.set_sensor_sonars_rear(sensor_sonars_rear); !r) return std::unexpected(r.error());
    return msg;
  };
  
  [[nodiscard]] double sensor_sonars_left() const noexcept {
    const uint16_t raw_sensor_sonars_left = detail::extract_le<uint16_t>(data_.data(), 16, 28);
    return static_cast<double>(raw_sensor_sonars_left) * 0.1 + 0;
  };
  
  [[nodiscard]] double sensor_sonars_middle() const noexcept {
    const uint16_t raw_sensor_sonars_middle = detail::extract_le<uint16_t>(data_.data(), 28, 40);
    return static_cast<double>(raw_sensor_sonars_middle) * 0.1 + 0;
  };
  
  [[nodiscard]] double sensor_sonars_right() const noexcept {
    const uint16_t raw_sensor_sonars_right = detail::extract_le<uint16_t>(data_.data(), 40, 52);
    return static_cast<double>(raw_sensor_sonars_right) * 0.1 + 0;
  };
  
  [[nodiscard]] double sensor_sonars_rear() const noexcept {
    const uint16_t raw_sensor_sonars_rear = detail::extract_le<uint16_t>(data_.data(), 52, 64);
    return static_cast<double>(raw_sensor_sonars_rear) * 0.1 + 0;
  };
  
  std::expected<void, CanError> set_sensor_sonars_left(double sensor_sonars_left) noexcept {
    if (sensor_sonars_left < 0f || sensor_sonars_left > 0f) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 16, 28, static_cast<uint16_t>((sensor_sonars_left - 0) / 0.1));
    return {};
  };
  
  std::expected<void, CanError> set_sensor_sonars_middle(double sensor_sonars_middle) noexcept {
    if (sensor_sonars_middle < 0f || sensor_sonars_middle > 0f) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 28, 40, static_cast<uint16_t>((sensor_sonars_middle - 0) / 0.1));
    return {};
  };
  
  std::expected<void, CanError> set_sensor_sonars_right(double sensor_sonars_right) noexcept {
    if (sensor_sonars_right < 0f || sensor_sonars_right > 0f) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 40, 52, static_cast<uint16_t>((sensor_sonars_right - 0) / 0.1));
    return {};
  };
  
  std::expected<void, CanError> set_sensor_sonars_rear(double sensor_sonars_rear) noexcept {
    if (sensor_sonars_rear < 0f || sensor_sonars_rear > 0f) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 52, 64, static_cast<uint16_t>((sensor_sonars_rear - 0) / 0.1));
    return {};
  };
  
  [[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept { return data_; }
  
  private:
  friend class SensorSonars;
  std::array<uint8_t, LEN> data_{};
};

class SensorSonarsMux1 {
  public:
  static constexpr std::size_t LEN = 8;
  
  [[nodiscard]] static std::expected<SensorSonarsMux1, CanError> create(
            double sensor_sonars_no_filt_left,
            double sensor_sonars_no_filt_middle,
            double sensor_sonars_no_filt_right,
            double sensor_sonars_no_filt_rear
        ) noexcept {
    SensorSonarsMux1 msg{};
    if (auto r = msg.set_sensor_sonars_no_filt_left(sensor_sonars_no_filt_left); !r) return std::unexpected(r.error());
    if (auto r = msg.set_sensor_sonars_no_filt_middle(sensor_sonars_no_filt_middle); !r) return std::unexpected(r.error());
    if (auto r = msg.set_sensor_sonars_no_filt_right(sensor_sonars_no_filt_right); !r) return std::unexpected(r.error());
    if (auto r = msg.set_sensor_sonars_no_filt_rear(sensor_sonars_no_filt_rear); !r) return std::unexpected(r.error());
    return msg;
  };
  
  [[nodiscard]] double sensor_sonars_no_filt_left() const noexcept {
    const uint16_t raw_sensor_sonars_no_filt_left = detail::extract_le<uint16_t>(data_.data(), 16, 28);
    return static_cast<double>(raw_sensor_sonars_no_filt_left) * 0.1 + 0;
  };
  
  [[nodiscard]] double sensor_sonars_no_filt_middle() const noexcept {
    const uint16_t raw_sensor_sonars_no_filt_middle = detail::extract_le<uint16_t>(data_.data(), 28, 40);
    return static_cast<double>(raw_sensor_sonars_no_filt_middle) * 0.1 + 0;
  };
  
  [[nodiscard]] double sensor_sonars_no_filt_right() const noexcept {
    const uint16_t raw_sensor_sonars_no_filt_right = detail::extract_le<uint16_t>(data_.data(), 40, 52);
    return static_cast<double>(raw_sensor_sonars_no_filt_right) * 0.1 + 0;
  };
  
  [[nodiscard]] double sensor_sonars_no_filt_rear() const noexcept {
    const uint16_t raw_sensor_sonars_no_filt_rear = detail::extract_le<uint16_t>(data_.data(), 52, 64);
    return static_cast<double>(raw_sensor_sonars_no_filt_rear) * 0.1 + 0;
  };
  
  std::expected<void, CanError> set_sensor_sonars_no_filt_left(double sensor_sonars_no_filt_left) noexcept {
    if (sensor_sonars_no_filt_left < 0f || sensor_sonars_no_filt_left > 0f) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 16, 28, static_cast<uint16_t>((sensor_sonars_no_filt_left - 0) / 0.1));
    return {};
  };
  
  std::expected<void, CanError> set_sensor_sonars_no_filt_middle(double sensor_sonars_no_filt_middle) noexcept {
    if (sensor_sonars_no_filt_middle < 0f || sensor_sonars_no_filt_middle > 0f) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 28, 40, static_cast<uint16_t>((sensor_sonars_no_filt_middle - 0) / 0.1));
    return {};
  };
  
  std::expected<void, CanError> set_sensor_sonars_no_filt_right(double sensor_sonars_no_filt_right) noexcept {
    if (sensor_sonars_no_filt_right < 0f || sensor_sonars_no_filt_right > 0f) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 40, 52, static_cast<uint16_t>((sensor_sonars_no_filt_right - 0) / 0.1));
    return {};
  };
  
  std::expected<void, CanError> set_sensor_sonars_no_filt_rear(double sensor_sonars_no_filt_rear) noexcept {
    if (sensor_sonars_no_filt_rear < 0f || sensor_sonars_no_filt_rear > 0f) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 52, 64, static_cast<uint16_t>((sensor_sonars_no_filt_rear - 0) / 0.1));
    return {};
  };
  
  [[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept { return data_; }
  
  private:
  friend class SensorSonars;
  std::array<uint8_t, LEN> data_{};
};

using SensorSonarsMux = std::variant<SensorSonarsMux0, SensorSonarsMux1>;

class SensorSonars {
  public:
  static constexpr uint16_t ID = 200;
  static constexpr std::size_t LEN = 8;
  
  [[nodiscard]] static std::expected<SensorSonars, CanError> try_from_frame(std::span<const uint8_t> frame) noexcept {
    if (frame.size() < LEN) return std::unexpected(CanError::InvalidPayloadSize);
    SensorSonars msg{};
    std::memcpy(msg.data_.data(), frame.data(), LEN);
    return msg;
  };
  
  [[nodiscard]] uint16_t sensor_sonars_err_count() const noexcept {
    const uint16_t raw_sensor_sonars_err_count = detail::extract_le<uint16_t>(data_.data(), 4, 16);
    return static_cast<uint16_t>(raw_sensor_sonars_err_count) * 1 + 0;
  };
  
  std::expected<void, CanError> set_sensor_sonars_err_count(uint16_t sensor_sonars_err_count) noexcept {
    if (sensor_sonars_err_count < 0 || sensor_sonars_err_count > 0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 4, 16, static_cast<uint16_t>((sensor_sonars_err_count - 0) / 1));
    return {};
  };
  
  [[nodiscard]] std::expected<SensorSonarsMux, CanError> mux() const noexcept {
    const uint8_t mux_raw = detail::extract_le<uint8_t>(data_.data(), 0, 4);
    switch (mux_raw) {
      case 0: {
        SensorSonarsMux0 inner{};
        std::memcpy(inner.data_.data(), data_.data(), LEN);
        return inner;
      };
      case 1: {
        SensorSonarsMux1 inner{};
        std::memcpy(inner.data_.data(), data_.data(), LEN);
        return inner;
      };
      default: return std::unexpected(CanError::UnknownMuxValue);
    };
  };
  
  void set_mux_0(const SensorSonarsMux0& value) noexcept {
    for (std::size_t i = 0; i < LEN; ++i) data_[i] |= value.data_[i];
    detail::insert_le<uint8_t>(data_.data(), 0, 4, static_cast<uint8_t>(0));
  };
  void set_mux_1(const SensorSonarsMux1& value) noexcept {
    for (std::size_t i = 0; i < LEN; ++i) data_[i] |= value.data_[i];
    detail::insert_le<uint8_t>(data_.data(), 0, 4, static_cast<uint8_t>(1));
  };
  
  [[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept { return data_; }
  
  private:
  std::array<uint8_t, LEN> data_{};
  ;
};

using CanMsg = std::variant<DriverHeartbeat, IoDebug, MotorCmd, MotorStatus, SensorSonars>;

[[nodiscard]]
inline std::expected<CanMsg, CanError>
parse_can(uint32_t id, std::span<const uint8_t> frame) noexcept {
  switch (id) {
    case DriverHeartbeat::ID:
     {
      auto r = DriverHeartbeat::try_from_frame(frame);
      if (!r) return std::unexpected(r.error());
      return CanMsg{std::move(*r)};
    };
    case IoDebug::ID:
     {
      auto r = IoDebug::try_from_frame(frame);
      if (!r) return std::unexpected(r.error());
      return CanMsg{std::move(*r)};
    };
    case MotorCmd::ID:
     {
      auto r = MotorCmd::try_from_frame(frame);
      if (!r) return std::unexpected(r.error());
      return CanMsg{std::move(*r)};
    };
    case MotorStatus::ID:
     {
      auto r = MotorStatus::try_from_frame(frame);
      if (!r) return std::unexpected(r.error());
      return CanMsg{std::move(*r)};
    };
    case SensorSonars::ID:
     {
      auto r = SensorSonars::try_from_frame(frame);
      if (!r) return std::unexpected(r.error());
      return CanMsg{std::move(*r)};
    };
    default: return std::unexpected(CanError::UnknownFrameId);
  };
};
