#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <span>
#include <variant>
#include <utility>
#include <cstring>

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

/**
 * DRIVER_HEARTBEAT
 * - ID: Standard 100 (0x64)
 * - Size: 1 bytes
 * - Transmitter: DRIVER
 * 
 * Sync message used to synchronize the controllers
 */
class DriverHeartbeatMsg {
  public:
  static constexpr uint16_t ID = 100;
  static constexpr std::size_t LEN = 1;
  
  [[nodiscard]] static std::expected<DriverHeartbeatMsg, CanError> create(
            DriverHeartbeatCmd driver_heartbeat_cmd
        ) noexcept {
    DriverHeartbeatMsg msg{};
    if (auto r = msg.set_driver_heartbeat_cmd(driver_heartbeat_cmd); !r) return std::unexpected(r.error());
    return msg;
  };
  
  [[nodiscard]] static std::expected<DriverHeartbeatMsg, CanError> try_from_frame(std::span<const uint8_t> frame) noexcept {
    if (frame.size() < LEN) return std::unexpected(CanError::InvalidPayloadSize);
    DriverHeartbeatMsg msg{};
    std::memcpy(msg.data_.data(), frame.data(), LEN);
    return msg;
  };
  
  /**
   * DRIVER_HEARTBEAT_cmd
   * - Min: 0
   * - Max: 0
   * - Unit: 
   * - Receivers: SENSOR, MOTOR
   * - Start bit: 0
   * - Size: 8 bits
   * - Factor: 1
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
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

/**
 * IO_DEBUG
 * - ID: Standard 500 (0x1F4)
 * - Size: 4 bytes
 * - Transmitter: IO
 */
class IoDebugMsg {
  public:
  static constexpr uint16_t ID = 500;
  static constexpr std::size_t LEN = 4;
  
  [[nodiscard]] static std::expected<IoDebugMsg, CanError> create(
            uint8_t io_debug_test_unsigned,
            IoDebugTestEnum io_debug_test_enum,
            int8_t io_debug_test_signed,
            double io_debug_test_float
        ) noexcept {
    IoDebugMsg msg{};
    if (auto r = msg.set_io_debug_test_unsigned(io_debug_test_unsigned); !r) return std::unexpected(r.error());
    if (auto r = msg.set_io_debug_test_enum(io_debug_test_enum); !r) return std::unexpected(r.error());
    if (auto r = msg.set_io_debug_test_signed(io_debug_test_signed); !r) return std::unexpected(r.error());
    if (auto r = msg.set_io_debug_test_float(io_debug_test_float); !r) return std::unexpected(r.error());
    return msg;
  };
  
  [[nodiscard]] static std::expected<IoDebugMsg, CanError> try_from_frame(std::span<const uint8_t> frame) noexcept {
    if (frame.size() < LEN) return std::unexpected(CanError::InvalidPayloadSize);
    IoDebugMsg msg{};
    std::memcpy(msg.data_.data(), frame.data(), LEN);
    return msg;
  };
  
  /**
   * IO_DEBUG_test_unsigned
   * - Min: 0
   * - Max: 0
   * - Unit: 
   * - Receivers: DBG
   * - Start bit: 0
   * - Size: 8 bits
   * - Factor: 1
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
  [[nodiscard]] uint8_t io_debug_test_unsigned() const noexcept {
    const uint8_t raw_io_debug_test_unsigned = detail::extract_le<uint8_t>(data_.data(), 0, 8);
    return static_cast<uint8_t>(raw_io_debug_test_unsigned) * 1 + 0;
  };
  
  /**
   * IO_DEBUG_test_enum
   * - Min: 0
   * - Max: 0
   * - Unit: 
   * - Receivers: DBG
   * - Start bit: 8
   * - Size: 8 bits
   * - Factor: 1
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
  [[nodiscard]] std::expected<IoDebugTestEnum, CanError> io_debug_test_enum() const noexcept {
    const uint8_t raw_io_debug_test_enum = detail::extract_le<uint8_t>(data_.data(), 8, 16);
    return io_debug_test_enum_from_raw(raw_io_debug_test_enum);
  };
  
  /**
   * IO_DEBUG_test_signed
   * - Min: 0
   * - Max: 0
   * - Unit: 
   * - Receivers: DBG
   * - Start bit: 16
   * - Size: 8 bits
   * - Factor: 1
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: signed
   */
  [[nodiscard]] int8_t io_debug_test_signed() const noexcept {
    const int8_t raw_io_debug_test_signed = detail::extract_le<int8_t>(data_.data(), 16, 24);
    return static_cast<int8_t>(raw_io_debug_test_signed) * 1 + 0;
  };
  
  /**
   * IO_DEBUG_test_float
   * - Min: 0
   * - Max: 0
   * - Unit: 
   * - Receivers: DBG
   * - Start bit: 24
   * - Size: 8 bits
   * - Factor: 0.5
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
  [[nodiscard]] double io_debug_test_float() const noexcept {
    const uint8_t raw_io_debug_test_float = detail::extract_le<uint8_t>(data_.data(), 24, 32);
    return static_cast<double>(raw_io_debug_test_float) * 0.5 + 0.0;
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
    if (io_debug_test_float < 0.0 || io_debug_test_float > 0.0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint8_t>(data_.data(), 24, 32, static_cast<uint8_t>((io_debug_test_float - 0.0) / 0.5));
    return {};
  };
  
  [[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept { return data_; }
  
  private:
  std::array<uint8_t, LEN> data_{};
};

/**
 * MOTOR_CMD
 * - ID: Standard 101 (0x65)
 * - Size: 1 bytes
 * - Transmitter: DRIVER
 */
class MotorCmdMsg {
  public:
  static constexpr uint16_t ID = 101;
  static constexpr std::size_t LEN = 1;
  
  [[nodiscard]] static std::expected<MotorCmdMsg, CanError> create(
            int8_t motor_cmd_steer,
            uint8_t motor_cmd_drive
        ) noexcept {
    MotorCmdMsg msg{};
    if (auto r = msg.set_motor_cmd_steer(motor_cmd_steer); !r) return std::unexpected(r.error());
    if (auto r = msg.set_motor_cmd_drive(motor_cmd_drive); !r) return std::unexpected(r.error());
    return msg;
  };
  
  [[nodiscard]] static std::expected<MotorCmdMsg, CanError> try_from_frame(std::span<const uint8_t> frame) noexcept {
    if (frame.size() < LEN) return std::unexpected(CanError::InvalidPayloadSize);
    MotorCmdMsg msg{};
    std::memcpy(msg.data_.data(), frame.data(), LEN);
    return msg;
  };
  
  /**
   * MOTOR_CMD_steer
   * - Min: -5
   * - Max: 5
   * - Unit: 
   * - Receivers: MOTOR
   * - Start bit: 0
   * - Size: 4 bits
   * - Factor: 1
   * - Offset: -5
   * - Byte order: LittleEndian
   * - Type: signed
   */
  [[nodiscard]] int8_t motor_cmd_steer() const noexcept {
    const int8_t raw_motor_cmd_steer = detail::extract_le<int8_t>(data_.data(), 0, 4);
    return static_cast<int8_t>(raw_motor_cmd_steer) * 1 + -5;
  };
  
  /**
   * MOTOR_CMD_drive
   * - Min: 0
   * - Max: 9
   * - Unit: 
   * - Receivers: MOTOR
   * - Start bit: 4
   * - Size: 4 bits
   * - Factor: 1
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
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

/**
 * MOTOR_STATUS
 * - ID: Standard 400 (0x190)
 * - Size: 3 bytes
 * - Transmitter: MOTOR
 */
class MotorStatusMsg {
  public:
  static constexpr uint16_t ID = 400;
  static constexpr std::size_t LEN = 3;
  
  [[nodiscard]] static std::expected<MotorStatusMsg, CanError> create(
            uint8_t motor_status_wheel_error,
            double motor_status_speed_kph
        ) noexcept {
    MotorStatusMsg msg{};
    if (auto r = msg.set_motor_status_wheel_error(motor_status_wheel_error); !r) return std::unexpected(r.error());
    if (auto r = msg.set_motor_status_speed_kph(motor_status_speed_kph); !r) return std::unexpected(r.error());
    return msg;
  };
  
  [[nodiscard]] static std::expected<MotorStatusMsg, CanError> try_from_frame(std::span<const uint8_t> frame) noexcept {
    if (frame.size() < LEN) return std::unexpected(CanError::InvalidPayloadSize);
    MotorStatusMsg msg{};
    std::memcpy(msg.data_.data(), frame.data(), LEN);
    return msg;
  };
  
  /**
   * MOTOR_STATUS_wheel_error
   * - Min: 0
   * - Max: 0
   * - Unit: 
   * - Receivers: DRIVER, IO
   * - Start bit: 0
   * - Size: 1 bits
   * - Factor: 1
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
  [[nodiscard]] uint8_t motor_status_wheel_error() const noexcept {
    const uint8_t raw_motor_status_wheel_error = detail::extract_le<uint8_t>(data_.data(), 0, 1);
    return static_cast<uint8_t>(raw_motor_status_wheel_error) * 1 + 0;
  };
  
  /**
   * MOTOR_STATUS_speed_kph
   * - Min: 0
   * - Max: 0
   * - Unit: kph
   * - Receivers: DRIVER, IO
   * - Start bit: 8
   * - Size: 16 bits
   * - Factor: 0.001
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
  [[nodiscard]] double motor_status_speed_kph() const noexcept {
    const uint16_t raw_motor_status_speed_kph = detail::extract_le<uint16_t>(data_.data(), 8, 24);
    return static_cast<double>(raw_motor_status_speed_kph) * 0.001 + 0.0;
  };
  
  std::expected<void, CanError> set_motor_status_wheel_error(uint8_t motor_status_wheel_error) noexcept {
    if (motor_status_wheel_error < 0 || motor_status_wheel_error > 0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint8_t>(data_.data(), 0, 1, static_cast<uint8_t>((motor_status_wheel_error - 0) / 1));
    return {};
  };
  
  std::expected<void, CanError> set_motor_status_speed_kph(double motor_status_speed_kph) noexcept {
    if (motor_status_speed_kph < 0.0 || motor_status_speed_kph > 0.0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 8, 24, static_cast<uint16_t>((motor_status_speed_kph - 0.0) / 0.001));
    return {};
  };
  
  [[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept { return data_; }
  
  private:
  std::array<uint8_t, LEN> data_{};
};

class SensorSonarsMsgMux0 {
  public:
  static constexpr std::size_t LEN = 8;
  
  [[nodiscard]] static std::expected<SensorSonarsMsgMux0, CanError> create(
            double sensor_sonars_left,
            double sensor_sonars_middle,
            double sensor_sonars_right,
            double sensor_sonars_rear
        ) noexcept {
    SensorSonarsMsgMux0 msg{};
    if (auto r = msg.set_sensor_sonars_left(sensor_sonars_left); !r) return std::unexpected(r.error());
    if (auto r = msg.set_sensor_sonars_middle(sensor_sonars_middle); !r) return std::unexpected(r.error());
    if (auto r = msg.set_sensor_sonars_right(sensor_sonars_right); !r) return std::unexpected(r.error());
    if (auto r = msg.set_sensor_sonars_rear(sensor_sonars_rear); !r) return std::unexpected(r.error());
    return msg;
  };
  
  /**
   * SENSOR_SONARS_left
   * - Min: 0
   * - Max: 0
   * - Unit: 
   * - Receivers: DRIVER, IO
   * - Start bit: 16
   * - Size: 12 bits
   * - Factor: 0.1
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
  [[nodiscard]] double sensor_sonars_left() const noexcept {
    const uint16_t raw_sensor_sonars_left = detail::extract_le<uint16_t>(data_.data(), 16, 28);
    return static_cast<double>(raw_sensor_sonars_left) * 0.1 + 0.0;
  };
  
  /**
   * SENSOR_SONARS_middle
   * - Min: 0
   * - Max: 0
   * - Unit: 
   * - Receivers: DRIVER, IO
   * - Start bit: 28
   * - Size: 12 bits
   * - Factor: 0.1
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
  [[nodiscard]] double sensor_sonars_middle() const noexcept {
    const uint16_t raw_sensor_sonars_middle = detail::extract_le<uint16_t>(data_.data(), 28, 40);
    return static_cast<double>(raw_sensor_sonars_middle) * 0.1 + 0.0;
  };
  
  /**
   * SENSOR_SONARS_right
   * - Min: 0
   * - Max: 0
   * - Unit: 
   * - Receivers: DRIVER, IO
   * - Start bit: 40
   * - Size: 12 bits
   * - Factor: 0.1
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
  [[nodiscard]] double sensor_sonars_right() const noexcept {
    const uint16_t raw_sensor_sonars_right = detail::extract_le<uint16_t>(data_.data(), 40, 52);
    return static_cast<double>(raw_sensor_sonars_right) * 0.1 + 0.0;
  };
  
  /**
   * SENSOR_SONARS_rear
   * - Min: 0
   * - Max: 0
   * - Unit: 
   * - Receivers: DRIVER, IO
   * - Start bit: 52
   * - Size: 12 bits
   * - Factor: 0.1
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
  [[nodiscard]] double sensor_sonars_rear() const noexcept {
    const uint16_t raw_sensor_sonars_rear = detail::extract_le<uint16_t>(data_.data(), 52, 64);
    return static_cast<double>(raw_sensor_sonars_rear) * 0.1 + 0.0;
  };
  
  std::expected<void, CanError> set_sensor_sonars_left(double sensor_sonars_left) noexcept {
    if (sensor_sonars_left < 0.0 || sensor_sonars_left > 0.0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 16, 28, static_cast<uint16_t>((sensor_sonars_left - 0.0) / 0.1));
    return {};
  };
  
  std::expected<void, CanError> set_sensor_sonars_middle(double sensor_sonars_middle) noexcept {
    if (sensor_sonars_middle < 0.0 || sensor_sonars_middle > 0.0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 28, 40, static_cast<uint16_t>((sensor_sonars_middle - 0.0) / 0.1));
    return {};
  };
  
  std::expected<void, CanError> set_sensor_sonars_right(double sensor_sonars_right) noexcept {
    if (sensor_sonars_right < 0.0 || sensor_sonars_right > 0.0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 40, 52, static_cast<uint16_t>((sensor_sonars_right - 0.0) / 0.1));
    return {};
  };
  
  std::expected<void, CanError> set_sensor_sonars_rear(double sensor_sonars_rear) noexcept {
    if (sensor_sonars_rear < 0.0 || sensor_sonars_rear > 0.0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 52, 64, static_cast<uint16_t>((sensor_sonars_rear - 0.0) / 0.1));
    return {};
  };
  
  [[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept { return data_; }
  
  private:
  friend class SensorSonarsMsg;
  std::array<uint8_t, LEN> data_{};
};

class SensorSonarsMsgMux1 {
  public:
  static constexpr std::size_t LEN = 8;
  
  [[nodiscard]] static std::expected<SensorSonarsMsgMux1, CanError> create(
            double sensor_sonars_no_filt_left,
            double sensor_sonars_no_filt_middle,
            double sensor_sonars_no_filt_right,
            double sensor_sonars_no_filt_rear
        ) noexcept {
    SensorSonarsMsgMux1 msg{};
    if (auto r = msg.set_sensor_sonars_no_filt_left(sensor_sonars_no_filt_left); !r) return std::unexpected(r.error());
    if (auto r = msg.set_sensor_sonars_no_filt_middle(sensor_sonars_no_filt_middle); !r) return std::unexpected(r.error());
    if (auto r = msg.set_sensor_sonars_no_filt_right(sensor_sonars_no_filt_right); !r) return std::unexpected(r.error());
    if (auto r = msg.set_sensor_sonars_no_filt_rear(sensor_sonars_no_filt_rear); !r) return std::unexpected(r.error());
    return msg;
  };
  
  /**
   * SENSOR_SONARS_no_filt_left
   * - Min: 0
   * - Max: 0
   * - Unit: 
   * - Receivers: DBG
   * - Start bit: 16
   * - Size: 12 bits
   * - Factor: 0.1
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
  [[nodiscard]] double sensor_sonars_no_filt_left() const noexcept {
    const uint16_t raw_sensor_sonars_no_filt_left = detail::extract_le<uint16_t>(data_.data(), 16, 28);
    return static_cast<double>(raw_sensor_sonars_no_filt_left) * 0.1 + 0.0;
  };
  
  /**
   * SENSOR_SONARS_no_filt_middle
   * - Min: 0
   * - Max: 0
   * - Unit: 
   * - Receivers: DBG
   * - Start bit: 28
   * - Size: 12 bits
   * - Factor: 0.1
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
  [[nodiscard]] double sensor_sonars_no_filt_middle() const noexcept {
    const uint16_t raw_sensor_sonars_no_filt_middle = detail::extract_le<uint16_t>(data_.data(), 28, 40);
    return static_cast<double>(raw_sensor_sonars_no_filt_middle) * 0.1 + 0.0;
  };
  
  /**
   * SENSOR_SONARS_no_filt_right
   * - Min: 0
   * - Max: 0
   * - Unit: 
   * - Receivers: DBG
   * - Start bit: 40
   * - Size: 12 bits
   * - Factor: 0.1
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
  [[nodiscard]] double sensor_sonars_no_filt_right() const noexcept {
    const uint16_t raw_sensor_sonars_no_filt_right = detail::extract_le<uint16_t>(data_.data(), 40, 52);
    return static_cast<double>(raw_sensor_sonars_no_filt_right) * 0.1 + 0.0;
  };
  
  /**
   * SENSOR_SONARS_no_filt_rear
   * - Min: 0
   * - Max: 0
   * - Unit: 
   * - Receivers: DBG
   * - Start bit: 52
   * - Size: 12 bits
   * - Factor: 0.1
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
  [[nodiscard]] double sensor_sonars_no_filt_rear() const noexcept {
    const uint16_t raw_sensor_sonars_no_filt_rear = detail::extract_le<uint16_t>(data_.data(), 52, 64);
    return static_cast<double>(raw_sensor_sonars_no_filt_rear) * 0.1 + 0.0;
  };
  
  std::expected<void, CanError> set_sensor_sonars_no_filt_left(double sensor_sonars_no_filt_left) noexcept {
    if (sensor_sonars_no_filt_left < 0.0 || sensor_sonars_no_filt_left > 0.0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 16, 28, static_cast<uint16_t>((sensor_sonars_no_filt_left - 0.0) / 0.1));
    return {};
  };
  
  std::expected<void, CanError> set_sensor_sonars_no_filt_middle(double sensor_sonars_no_filt_middle) noexcept {
    if (sensor_sonars_no_filt_middle < 0.0 || sensor_sonars_no_filt_middle > 0.0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 28, 40, static_cast<uint16_t>((sensor_sonars_no_filt_middle - 0.0) / 0.1));
    return {};
  };
  
  std::expected<void, CanError> set_sensor_sonars_no_filt_right(double sensor_sonars_no_filt_right) noexcept {
    if (sensor_sonars_no_filt_right < 0.0 || sensor_sonars_no_filt_right > 0.0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 40, 52, static_cast<uint16_t>((sensor_sonars_no_filt_right - 0.0) / 0.1));
    return {};
  };
  
  std::expected<void, CanError> set_sensor_sonars_no_filt_rear(double sensor_sonars_no_filt_rear) noexcept {
    if (sensor_sonars_no_filt_rear < 0.0 || sensor_sonars_no_filt_rear > 0.0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 52, 64, static_cast<uint16_t>((sensor_sonars_no_filt_rear - 0.0) / 0.1));
    return {};
  };
  
  [[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept { return data_; }
  
  private:
  friend class SensorSonarsMsg;
  std::array<uint8_t, LEN> data_{};
};

using SensorSonarsMsgMux = std::variant<SensorSonarsMsgMux0, SensorSonarsMsgMux1>;

/**
 * SENSOR_SONARS
 * - ID: Standard 200 (0xC8)
 * - Size: 8 bytes
 * - Transmitter: SENSOR
 */
class SensorSonarsMsg {
  public:
  static constexpr uint16_t ID = 200;
  static constexpr std::size_t LEN = 8;
  
  [[nodiscard]] static std::expected<SensorSonarsMsg, CanError> try_from_frame(std::span<const uint8_t> frame) noexcept {
    if (frame.size() < LEN) return std::unexpected(CanError::InvalidPayloadSize);
    SensorSonarsMsg msg{};
    std::memcpy(msg.data_.data(), frame.data(), LEN);
    return msg;
  };
  
  /**
   * SENSOR_SONARS_err_count
   * - Min: 0
   * - Max: 0
   * - Unit: 
   * - Receivers: DRIVER, IO
   * - Start bit: 4
   * - Size: 12 bits
   * - Factor: 1
   * - Offset: 0
   * - Byte order: LittleEndian
   * - Type: unsigned
   */
  [[nodiscard]] uint16_t sensor_sonars_err_count() const noexcept {
    const uint16_t raw_sensor_sonars_err_count = detail::extract_le<uint16_t>(data_.data(), 4, 16);
    return static_cast<uint16_t>(raw_sensor_sonars_err_count) * 1 + 0;
  };
  
  std::expected<void, CanError> set_sensor_sonars_err_count(uint16_t sensor_sonars_err_count) noexcept {
    if (sensor_sonars_err_count < 0 || sensor_sonars_err_count > 0) return std::unexpected(CanError::ValueOutOfRange);
    detail::insert_le<uint16_t>(data_.data(), 4, 16, static_cast<uint16_t>((sensor_sonars_err_count - 0) / 1));
    return {};
  };
  
  [[nodiscard]] std::expected<SensorSonarsMsgMux, CanError> mux() const noexcept {
    const uint8_t mux_raw = detail::extract_le<uint8_t>(data_.data(), 0, 4);
    switch (mux_raw) {
      case 0: {
        SensorSonarsMsgMux0 inner{};
        std::memcpy(inner.data_.data(), data_.data(), LEN);
        return inner;
      };
      case 1: {
        SensorSonarsMsgMux1 inner{};
        std::memcpy(inner.data_.data(), data_.data(), LEN);
        return inner;
      };
      default: return std::unexpected(CanError::UnknownMuxValue);
    };
  };
  
  void set_mux_0(const SensorSonarsMsgMux0& value) noexcept {
    for (std::size_t i = 0; i < LEN; ++i) data_[i] |= value.data_[i];
    detail::insert_le<uint8_t>(data_.data(), 0, 4, static_cast<uint8_t>(0));
  };
  void set_mux_1(const SensorSonarsMsgMux1& value) noexcept {
    for (std::size_t i = 0; i < LEN; ++i) data_[i] |= value.data_[i];
    detail::insert_le<uint8_t>(data_.data(), 0, 4, static_cast<uint8_t>(1));
  };
  
  [[nodiscard]] std::array<uint8_t, LEN> encode() const noexcept { return data_; }
  
  private:
  std::array<uint8_t, LEN> data_{};
  ;
};

using CanMsg = std::variant<DriverHeartbeatMsg, IoDebugMsg, MotorCmdMsg, MotorStatusMsg, SensorSonarsMsg>;

[[nodiscard]]
inline std::expected<CanMsg, CanError>
parse_can(uint32_t id, std::span<const uint8_t> frame) noexcept {
  switch (id) {
    case DriverHeartbeatMsg::ID:
     {
      auto r = DriverHeartbeatMsg::try_from_frame(frame);
      if (!r) return std::unexpected(r.error());
      return CanMsg{std::move(*r)};
    };
    case IoDebugMsg::ID:
     {
      auto r = IoDebugMsg::try_from_frame(frame);
      if (!r) return std::unexpected(r.error());
      return CanMsg{std::move(*r)};
    };
    case MotorCmdMsg::ID:
     {
      auto r = MotorCmdMsg::try_from_frame(frame);
      if (!r) return std::unexpected(r.error());
      return CanMsg{std::move(*r)};
    };
    case MotorStatusMsg::ID:
     {
      auto r = MotorStatusMsg::try_from_frame(frame);
      if (!r) return std::unexpected(r.error());
      return CanMsg{std::move(*r)};
    };
    case SensorSonarsMsg::ID:
     {
      auto r = SensorSonarsMsg::try_from_frame(frame);
      if (!r) return std::unexpected(r.error());
      return CanMsg{std::move(*r)};
    };
    default: return std::unexpected(CanError::UnknownFrameId);
  };
};
