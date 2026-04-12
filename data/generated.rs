use embedded_can::{Frame, Id, StandardId, ExtendedId};
use bitvec::prelude::*;
#[derive(Debug, Clone)]
pub enum CanError {
    UnknownFrameId,
    UnknownMuxValue,
    InvalidPayloadSize,
    ValueOutOfRange,
    IvalidEnumValue,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverHeartbeatCmd {
    Reboot,
    Sync,
    Noop,
    _Other(u8),
}
impl From<u8> for DriverHeartbeatCmd {
    fn from(val: u8) -> Self {
        match val {
            2u8 => DriverHeartbeatCmd::Reboot,
            1u8 => DriverHeartbeatCmd::Sync,
            0u8 => DriverHeartbeatCmd::Noop,
            _ => DriverHeartbeatCmd::_Other(val),
        }
    }
}
impl From<DriverHeartbeatCmd> for u8 {
    fn from(val: DriverHeartbeatCmd) -> Self {
        match val {
            DriverHeartbeatCmd::Reboot => 2u8,
            DriverHeartbeatCmd::Sync => 1u8,
            DriverHeartbeatCmd::Noop => 0u8,
            DriverHeartbeatCmd::_Other(v) => v,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoDebugTestEnum {
    Two,
    One,
    _Other(u8),
}
impl From<u8> for IoDebugTestEnum {
    fn from(val: u8) -> Self {
        match val {
            2u8 => IoDebugTestEnum::Two,
            1u8 => IoDebugTestEnum::One,
            _ => IoDebugTestEnum::_Other(val),
        }
    }
}
impl From<IoDebugTestEnum> for u8 {
    fn from(val: IoDebugTestEnum) -> Self {
        match val {
            IoDebugTestEnum::Two => 2u8,
            IoDebugTestEnum::One => 1u8,
            IoDebugTestEnum::_Other(v) => v,
        }
    }
}
pub trait CanMessage<const LEN: usize>: Sized {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError>;
    fn encode(&self) -> [u8; LEN];
}
#[derive(Debug, Clone)]
pub enum Msg {
    DriverHeartbeat(DriverHeartbeat),
    IoDebug(IoDebug),
    MotorCmd(MotorCmd),
    MotorStatus(MotorStatus),
    SensorSonars(SensorSonars),
}
impl Msg {
    fn try_from(frame: &impl Frame) -> Result<Self, CanError> {
        let result = match frame.id() {
            DriverHeartbeat::ID => {
                Msg::DriverHeartbeat(DriverHeartbeat::try_from_frame(frame)?)
            }
            IoDebug::ID => Msg::IoDebug(IoDebug::try_from_frame(frame)?),
            MotorCmd::ID => Msg::MotorCmd(MotorCmd::try_from_frame(frame)?),
            MotorStatus::ID => Msg::MotorStatus(MotorStatus::try_from_frame(frame)?),
            SensorSonars::ID => Msg::SensorSonars(SensorSonars::try_from_frame(frame)?),
            _ => return Err(CanError::UnknownFrameId),
        };
        Ok(result)
    }
}
///DRIVER_HEARTBEAT
///- ID: Standard 100 (0x64)
///- Size: 1 bytes
///- Transmitter: DRIVER
///
///Sync message used to synchronize the controllers
#[derive(Debug, Clone)]
pub struct DriverHeartbeat {
    data: [u8; 1usize],
}
impl DriverHeartbeat {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(100u16) });
    pub const LEN: usize = 1usize;
    pub fn new(driver_heartbeat_cmd: DriverHeartbeatCmd) -> Result<Self, CanError> {
        let mut msg = Self { data: [0u8; Self::LEN] };
        msg.set_driver_heartbeat_cmd(driver_heartbeat_cmd)?;
        Ok(msg)
    }
    ///DRIVER_HEARTBEAT_cmd
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: SENSOR, MOTOR
    ///- Start bit: 0
    ///- Size: 8 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn driver_heartbeat_cmd(&self) -> DriverHeartbeatCmd {
        let data = &self.data;
        let raw_driver_heartbeat_cmd = data
            .view_bits::<Lsb0>()[0usize..8usize]
            .load_le::<u8>();
        DriverHeartbeatCmd::from(raw_driver_heartbeat_cmd as u8)
    }
    pub fn set_driver_heartbeat_cmd(
        &mut self,
        value: DriverHeartbeatCmd,
    ) -> Result<(), CanError> {
        let data = &mut self.data;
        let driver_heartbeat_cmd = value;
        data.view_bits_mut::<Lsb0>()[0usize..8usize]
            .store_le(u8::from(driver_heartbeat_cmd));
        Ok(())
    }
}
impl CanMessage<{ Self::LEN }> for DriverHeartbeat {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        if data.len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        let mut buf = [0u8; 1usize];
        buf.copy_from_slice(&data[..1usize]);
        Ok(Self { data: buf })
    }
    fn encode(&self) -> [u8; Self::LEN] {
        self.data
    }
}
///IO_DEBUG
///- ID: Standard 500 (0x1F4)
///- Size: 4 bytes
///- Transmitter: IO
#[derive(Debug, Clone)]
pub struct IoDebug {
    data: [u8; 4usize],
}
impl IoDebug {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(500u16) });
    pub const LEN: usize = 4usize;
    pub fn new(
        io_debug_test_unsigned: u8,
        io_debug_test_enum: IoDebugTestEnum,
        io_debug_test_signed: i8,
        io_debug_test_float: f64,
    ) -> Result<Self, CanError> {
        let mut msg = Self { data: [0u8; Self::LEN] };
        msg.set_io_debug_test_unsigned(io_debug_test_unsigned)?;
        msg.set_io_debug_test_enum(io_debug_test_enum)?;
        msg.set_io_debug_test_signed(io_debug_test_signed)?;
        msg.set_io_debug_test_float(io_debug_test_float)?;
        Ok(msg)
    }
    ///IO_DEBUG_test_unsigned
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: DBG
    ///- Start bit: 0
    ///- Size: 8 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn io_debug_test_unsigned(&self) -> u8 {
        let data = &self.data;
        let raw_io_debug_test_unsigned = data
            .view_bits::<Lsb0>()[0usize..8usize]
            .load_le::<u8>();
        (raw_io_debug_test_unsigned) * (1u8) + (0u8)
    }
    ///IO_DEBUG_test_enum
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: DBG
    ///- Start bit: 8
    ///- Size: 8 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn io_debug_test_enum(&self) -> IoDebugTestEnum {
        let data = &self.data;
        let raw_io_debug_test_enum = data
            .view_bits::<Lsb0>()[8usize..16usize]
            .load_le::<u8>();
        IoDebugTestEnum::from(raw_io_debug_test_enum as u8)
    }
    ///IO_DEBUG_test_signed
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: DBG
    ///- Start bit: 16
    ///- Size: 8 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: signed
    pub fn io_debug_test_signed(&self) -> i8 {
        let data = &self.data;
        let raw_io_debug_test_signed = data
            .view_bits::<Lsb0>()[16usize..24usize]
            .load_le::<i8>();
        (raw_io_debug_test_signed) * (1i8) + (0i8)
    }
    ///IO_DEBUG_test_float
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: DBG
    ///- Start bit: 24
    ///- Size: 8 bits
    ///- Factor: 0.5
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn io_debug_test_float(&self) -> f64 {
        let data = &self.data;
        let raw_io_debug_test_float = data
            .view_bits::<Lsb0>()[24usize..32usize]
            .load_le::<u8>();
        (raw_io_debug_test_float as f64) * (0.5f64) + (0f64)
    }
    pub fn set_io_debug_test_unsigned(&mut self, value: u8) -> Result<(), CanError> {
        let data = &mut self.data;
        let io_debug_test_unsigned = value;
        data.view_bits_mut::<Lsb0>()[0usize..8usize]
            .store_le((io_debug_test_unsigned - (0u8)) / (1u8));
        Ok(())
    }
    pub fn set_io_debug_test_enum(
        &mut self,
        value: IoDebugTestEnum,
    ) -> Result<(), CanError> {
        let data = &mut self.data;
        let io_debug_test_enum = value;
        data.view_bits_mut::<Lsb0>()[8usize..16usize]
            .store_le(u8::from(io_debug_test_enum));
        Ok(())
    }
    pub fn set_io_debug_test_signed(&mut self, value: i8) -> Result<(), CanError> {
        let data = &mut self.data;
        let io_debug_test_signed = value;
        data.view_bits_mut::<Lsb0>()[16usize..24usize]
            .store_le((io_debug_test_signed - (0i8)) / (1i8));
        Ok(())
    }
    pub fn set_io_debug_test_float(&mut self, value: f64) -> Result<(), CanError> {
        let data = &mut self.data;
        let io_debug_test_float = value;
        data.view_bits_mut::<Lsb0>()[24usize..32usize]
            .store_le(((io_debug_test_float - (0f64)) / (0.5f64)) as u8);
        Ok(())
    }
}
impl CanMessage<{ Self::LEN }> for IoDebug {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        if data.len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        let mut buf = [0u8; 4usize];
        buf.copy_from_slice(&data[..4usize]);
        Ok(Self { data: buf })
    }
    fn encode(&self) -> [u8; Self::LEN] {
        self.data
    }
}
///MOTOR_CMD
///- ID: Standard 101 (0x65)
///- Size: 1 bytes
///- Transmitter: DRIVER
#[derive(Debug, Clone)]
pub struct MotorCmd {
    data: [u8; 1usize],
}
impl MotorCmd {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(101u16) });
    pub const LEN: usize = 1usize;
    pub fn new(motor_cmd_steer: i8, motor_cmd_drive: u8) -> Result<Self, CanError> {
        let mut msg = Self { data: [0u8; Self::LEN] };
        msg.set_motor_cmd_steer(motor_cmd_steer)?;
        msg.set_motor_cmd_drive(motor_cmd_drive)?;
        Ok(msg)
    }
    ///MOTOR_CMD_steer
    ///- Min: -5
    ///- Max: 5
    ///- Unit:
    ///- Receivers: MOTOR
    ///- Start bit: 0
    ///- Size: 4 bits
    ///- Factor: 1
    ///- Offset: -5
    ///- Byte order: LittleEndian
    ///- Type: signed
    pub fn motor_cmd_steer(&self) -> i8 {
        let data = &self.data;
        let raw_motor_cmd_steer = data
            .view_bits::<Lsb0>()[0usize..4usize]
            .load_le::<i8>();
        (raw_motor_cmd_steer) * (1i8) + (-5i8)
    }
    ///MOTOR_CMD_drive
    ///- Min: 0
    ///- Max: 9
    ///- Unit:
    ///- Receivers: MOTOR
    ///- Start bit: 4
    ///- Size: 4 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn motor_cmd_drive(&self) -> u8 {
        let data = &self.data;
        let raw_motor_cmd_drive = data
            .view_bits::<Lsb0>()[4usize..8usize]
            .load_le::<u8>();
        (raw_motor_cmd_drive) * (1u8) + (0u8)
    }
    pub fn set_motor_cmd_steer(&mut self, value: i8) -> Result<(), CanError> {
        if value < -5i8 || value > 5i8 {
            return Err(CanError::ValueOutOfRange);
        }
        let data = &mut self.data;
        let motor_cmd_steer = value;
        data.view_bits_mut::<Lsb0>()[0usize..4usize]
            .store_le((motor_cmd_steer - (-5i8)) / (1i8));
        Ok(())
    }
    pub fn set_motor_cmd_drive(&mut self, value: u8) -> Result<(), CanError> {
        if value < 0u8 || value > 9u8 {
            return Err(CanError::ValueOutOfRange);
        }
        let data = &mut self.data;
        let motor_cmd_drive = value;
        data.view_bits_mut::<Lsb0>()[4usize..8usize]
            .store_le((motor_cmd_drive - (0u8)) / (1u8));
        Ok(())
    }
}
impl CanMessage<{ Self::LEN }> for MotorCmd {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        if data.len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        let mut buf = [0u8; 1usize];
        buf.copy_from_slice(&data[..1usize]);
        Ok(Self { data: buf })
    }
    fn encode(&self) -> [u8; Self::LEN] {
        self.data
    }
}
///MOTOR_STATUS
///- ID: Standard 400 (0x190)
///- Size: 3 bytes
///- Transmitter: MOTOR
#[derive(Debug, Clone)]
pub struct MotorStatus {
    data: [u8; 3usize],
}
impl MotorStatus {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(400u16) });
    pub const LEN: usize = 3usize;
    pub fn new(
        motor_status_wheel_error: u8,
        motor_status_speed_kph: f64,
    ) -> Result<Self, CanError> {
        let mut msg = Self { data: [0u8; Self::LEN] };
        msg.set_motor_status_wheel_error(motor_status_wheel_error)?;
        msg.set_motor_status_speed_kph(motor_status_speed_kph)?;
        Ok(msg)
    }
    ///MOTOR_STATUS_wheel_error
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: DRIVER, IO
    ///- Start bit: 0
    ///- Size: 1 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn motor_status_wheel_error(&self) -> u8 {
        let data = &self.data;
        let raw_motor_status_wheel_error = data
            .view_bits::<Lsb0>()[0usize..1usize]
            .load_le::<u8>();
        (raw_motor_status_wheel_error) * (1u8) + (0u8)
    }
    ///MOTOR_STATUS_speed_kph
    ///- Min: 0
    ///- Max: 0
    ///- Unit: kph
    ///- Receivers: DRIVER, IO
    ///- Start bit: 8
    ///- Size: 16 bits
    ///- Factor: 0.001
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn motor_status_speed_kph(&self) -> f64 {
        let data = &self.data;
        let raw_motor_status_speed_kph = data
            .view_bits::<Lsb0>()[8usize..24usize]
            .load_le::<u16>();
        (raw_motor_status_speed_kph as f64) * (0.001f64) + (0f64)
    }
    pub fn set_motor_status_wheel_error(&mut self, value: u8) -> Result<(), CanError> {
        let data = &mut self.data;
        let motor_status_wheel_error = value;
        data.view_bits_mut::<Lsb0>()[0usize..1usize]
            .store_le((motor_status_wheel_error - (0u8)) / (1u8));
        Ok(())
    }
    pub fn set_motor_status_speed_kph(&mut self, value: f64) -> Result<(), CanError> {
        let data = &mut self.data;
        let motor_status_speed_kph = value;
        data.view_bits_mut::<Lsb0>()[8usize..24usize]
            .store_le(((motor_status_speed_kph - (0f64)) / (0.001f64)) as u16);
        Ok(())
    }
}
impl CanMessage<{ Self::LEN }> for MotorStatus {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        if data.len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        let mut buf = [0u8; 3usize];
        buf.copy_from_slice(&data[..3usize]);
        Ok(Self { data: buf })
    }
    fn encode(&self) -> [u8; Self::LEN] {
        self.data
    }
}
#[derive(Debug, Clone)]
pub enum SensorSonarsMux {
    V0,
    V1,
}
///SENSOR_SONARS
///- ID: Standard 200 (0xC8)
///- Size: 8 bytes
///- Transmitter: SENSOR
#[derive(Debug, Clone)]
pub struct SensorSonars {
    data: [u8; 8usize],
}
impl SensorSonars {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(200u16) });
    pub const LEN: usize = 8usize;
    pub fn new(
        sensor_sonars_mux: u8,
        sensor_sonars_err_count: u16,
        sensor_sonars_left: f64,
        sensor_sonars_middle: f64,
        sensor_sonars_right: f64,
        sensor_sonars_rear: f64,
        sensor_sonars_no_filt_left: f64,
        sensor_sonars_no_filt_middle: f64,
        sensor_sonars_no_filt_right: f64,
        sensor_sonars_no_filt_rear: f64,
    ) -> Result<Self, CanError> {
        let mut msg = Self { data: [0u8; 8usize] };
        msg.set_sensor_sonars_mux(sensor_sonars_mux)?;
        msg.set_sensor_sonars_err_count(sensor_sonars_err_count)?;
        msg.set_sensor_sonars_left(sensor_sonars_left)?;
        msg.set_sensor_sonars_middle(sensor_sonars_middle)?;
        msg.set_sensor_sonars_right(sensor_sonars_right)?;
        msg.set_sensor_sonars_rear(sensor_sonars_rear)?;
        msg.set_sensor_sonars_no_filt_left(sensor_sonars_no_filt_left)?;
        msg.set_sensor_sonars_no_filt_middle(sensor_sonars_no_filt_middle)?;
        msg.set_sensor_sonars_no_filt_right(sensor_sonars_no_filt_right)?;
        msg.set_sensor_sonars_no_filt_rear(sensor_sonars_no_filt_rear)?;
        Ok(msg)
    }
    pub fn mux(&self) -> Result<SensorSonarsMux, CanError> {
        let data = &self.data;
        let raw_sensor_sonars_mux = data
            .view_bits::<Lsb0>()[0usize..4usize]
            .load_le::<u8>();
        let val = (raw_sensor_sonars_mux) * (1u8) + (0u8);
        match val {
            0u8 => Ok(SensorSonarsMux::V0),
            1u8 => Ok(SensorSonarsMux::V1),
            _ => Err(CanError::UnknownMuxValue),
        }
    }
    pub fn set_mux(&mut self, mux: SensorSonarsMux) {
        let data = &mut self.data;
        match mux {
            SensorSonarsMux::V0 => {
                data.view_bits_mut::<Lsb0>()[0usize..4usize].store_le(0u64 as u8);
            }
            SensorSonarsMux::V1 => {
                data.view_bits_mut::<Lsb0>()[0usize..4usize].store_le(1u64 as u8);
            }
        }
    }
    ///SENSOR_SONARS_mux
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: DRIVER, IO
    ///- Start bit: 0
    ///- Size: 4 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn sensor_sonars_mux(&self) -> u8 {
        let data = &self.data;
        let raw_sensor_sonars_mux = data
            .view_bits::<Lsb0>()[0usize..4usize]
            .load_le::<u8>();
        (raw_sensor_sonars_mux) * (1u8) + (0u8)
    }
    ///SENSOR_SONARS_err_count
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: DRIVER, IO
    ///- Start bit: 4
    ///- Size: 12 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn sensor_sonars_err_count(&self) -> u16 {
        let data = &self.data;
        let raw_sensor_sonars_err_count = data
            .view_bits::<Lsb0>()[4usize..16usize]
            .load_le::<u16>();
        (raw_sensor_sonars_err_count) * (1u16) + (0u16)
    }
    ///SENSOR_SONARS_left
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: DRIVER, IO
    ///- Start bit: 16
    ///- Size: 12 bits
    ///- Factor: 0.1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn sensor_sonars_left(&self) -> f64 {
        let data = &self.data;
        let raw_sensor_sonars_left = data
            .view_bits::<Lsb0>()[16usize..28usize]
            .load_le::<u16>();
        (raw_sensor_sonars_left as f64) * (0.1f64) + (0f64)
    }
    ///SENSOR_SONARS_middle
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: DRIVER, IO
    ///- Start bit: 28
    ///- Size: 12 bits
    ///- Factor: 0.1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn sensor_sonars_middle(&self) -> f64 {
        let data = &self.data;
        let raw_sensor_sonars_middle = data
            .view_bits::<Lsb0>()[28usize..40usize]
            .load_le::<u16>();
        (raw_sensor_sonars_middle as f64) * (0.1f64) + (0f64)
    }
    ///SENSOR_SONARS_right
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: DRIVER, IO
    ///- Start bit: 40
    ///- Size: 12 bits
    ///- Factor: 0.1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn sensor_sonars_right(&self) -> f64 {
        let data = &self.data;
        let raw_sensor_sonars_right = data
            .view_bits::<Lsb0>()[40usize..52usize]
            .load_le::<u16>();
        (raw_sensor_sonars_right as f64) * (0.1f64) + (0f64)
    }
    ///SENSOR_SONARS_rear
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: DRIVER, IO
    ///- Start bit: 52
    ///- Size: 12 bits
    ///- Factor: 0.1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn sensor_sonars_rear(&self) -> f64 {
        let data = &self.data;
        let raw_sensor_sonars_rear = data
            .view_bits::<Lsb0>()[52usize..64usize]
            .load_le::<u16>();
        (raw_sensor_sonars_rear as f64) * (0.1f64) + (0f64)
    }
    ///SENSOR_SONARS_no_filt_left
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: DBG
    ///- Start bit: 16
    ///- Size: 12 bits
    ///- Factor: 0.1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn sensor_sonars_no_filt_left(&self) -> f64 {
        let data = &self.data;
        let raw_sensor_sonars_no_filt_left = data
            .view_bits::<Lsb0>()[16usize..28usize]
            .load_le::<u16>();
        (raw_sensor_sonars_no_filt_left as f64) * (0.1f64) + (0f64)
    }
    ///SENSOR_SONARS_no_filt_middle
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: DBG
    ///- Start bit: 28
    ///- Size: 12 bits
    ///- Factor: 0.1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn sensor_sonars_no_filt_middle(&self) -> f64 {
        let data = &self.data;
        let raw_sensor_sonars_no_filt_middle = data
            .view_bits::<Lsb0>()[28usize..40usize]
            .load_le::<u16>();
        (raw_sensor_sonars_no_filt_middle as f64) * (0.1f64) + (0f64)
    }
    ///SENSOR_SONARS_no_filt_right
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: DBG
    ///- Start bit: 40
    ///- Size: 12 bits
    ///- Factor: 0.1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn sensor_sonars_no_filt_right(&self) -> f64 {
        let data = &self.data;
        let raw_sensor_sonars_no_filt_right = data
            .view_bits::<Lsb0>()[40usize..52usize]
            .load_le::<u16>();
        (raw_sensor_sonars_no_filt_right as f64) * (0.1f64) + (0f64)
    }
    ///SENSOR_SONARS_no_filt_rear
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: DBG
    ///- Start bit: 52
    ///- Size: 12 bits
    ///- Factor: 0.1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn sensor_sonars_no_filt_rear(&self) -> f64 {
        let data = &self.data;
        let raw_sensor_sonars_no_filt_rear = data
            .view_bits::<Lsb0>()[52usize..64usize]
            .load_le::<u16>();
        (raw_sensor_sonars_no_filt_rear as f64) * (0.1f64) + (0f64)
    }
    pub fn set_sensor_sonars_mux(&mut self, value: u8) -> Result<(), CanError> {
        let data = &mut self.data;
        let sensor_sonars_mux = value;
        data.view_bits_mut::<Lsb0>()[0usize..4usize]
            .store_le((sensor_sonars_mux - (0u8)) / (1u8));
        Ok(())
    }
    pub fn set_sensor_sonars_err_count(&mut self, value: u16) -> Result<(), CanError> {
        let data = &mut self.data;
        let sensor_sonars_err_count = value;
        data.view_bits_mut::<Lsb0>()[4usize..16usize]
            .store_le((sensor_sonars_err_count - (0u16)) / (1u16));
        Ok(())
    }
    pub fn set_sensor_sonars_left(&mut self, value: f64) -> Result<(), CanError> {
        let data = &mut self.data;
        let sensor_sonars_left = value;
        data.view_bits_mut::<Lsb0>()[16usize..28usize]
            .store_le(((sensor_sonars_left - (0f64)) / (0.1f64)) as u16);
        Ok(())
    }
    pub fn set_sensor_sonars_middle(&mut self, value: f64) -> Result<(), CanError> {
        let data = &mut self.data;
        let sensor_sonars_middle = value;
        data.view_bits_mut::<Lsb0>()[28usize..40usize]
            .store_le(((sensor_sonars_middle - (0f64)) / (0.1f64)) as u16);
        Ok(())
    }
    pub fn set_sensor_sonars_right(&mut self, value: f64) -> Result<(), CanError> {
        let data = &mut self.data;
        let sensor_sonars_right = value;
        data.view_bits_mut::<Lsb0>()[40usize..52usize]
            .store_le(((sensor_sonars_right - (0f64)) / (0.1f64)) as u16);
        Ok(())
    }
    pub fn set_sensor_sonars_rear(&mut self, value: f64) -> Result<(), CanError> {
        let data = &mut self.data;
        let sensor_sonars_rear = value;
        data.view_bits_mut::<Lsb0>()[52usize..64usize]
            .store_le(((sensor_sonars_rear - (0f64)) / (0.1f64)) as u16);
        Ok(())
    }
    pub fn set_sensor_sonars_no_filt_left(
        &mut self,
        value: f64,
    ) -> Result<(), CanError> {
        let data = &mut self.data;
        let sensor_sonars_no_filt_left = value;
        data.view_bits_mut::<Lsb0>()[16usize..28usize]
            .store_le(((sensor_sonars_no_filt_left - (0f64)) / (0.1f64)) as u16);
        Ok(())
    }
    pub fn set_sensor_sonars_no_filt_middle(
        &mut self,
        value: f64,
    ) -> Result<(), CanError> {
        let data = &mut self.data;
        let sensor_sonars_no_filt_middle = value;
        data.view_bits_mut::<Lsb0>()[28usize..40usize]
            .store_le(((sensor_sonars_no_filt_middle - (0f64)) / (0.1f64)) as u16);
        Ok(())
    }
    pub fn set_sensor_sonars_no_filt_right(
        &mut self,
        value: f64,
    ) -> Result<(), CanError> {
        let data = &mut self.data;
        let sensor_sonars_no_filt_right = value;
        data.view_bits_mut::<Lsb0>()[40usize..52usize]
            .store_le(((sensor_sonars_no_filt_right - (0f64)) / (0.1f64)) as u16);
        Ok(())
    }
    pub fn set_sensor_sonars_no_filt_rear(
        &mut self,
        value: f64,
    ) -> Result<(), CanError> {
        let data = &mut self.data;
        let sensor_sonars_no_filt_rear = value;
        data.view_bits_mut::<Lsb0>()[52usize..64usize]
            .store_le(((sensor_sonars_no_filt_rear - (0f64)) / (0.1f64)) as u16);
        Ok(())
    }
}
impl CanMessage<{ 8usize }> for SensorSonars {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        if data.len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        let mut buf = [0u8; 8usize];
        buf.copy_from_slice(&data[..8usize]);
        Ok(Self { data: buf })
    }
    fn encode(&self) -> [u8; 8usize] {
        self.data
    }
}
