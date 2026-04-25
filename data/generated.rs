use embedded_can::{Frame, Id, StandardId, ExtendedId};
use bitvec::prelude::*;
use core::ops::BitOr;
use core::convert::TryFrom;
#[derive(Debug, Clone)]
pub enum CanError {
    UnknownFrameId,
    UnknownMuxValue,
    InvalidPayloadSize,
    InvalidFrameId,
    ValueOutOfRange,
    InvalidEnumValue,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverHeartbeatCmdEnum {
    Reboot,
    Sync,
    Noop,
    _Other(u8),
}
impl From<u8> for DriverHeartbeatCmdEnum {
    fn from(val: u8) -> Self {
        match val {
            2u8 => DriverHeartbeatCmdEnum::Reboot,
            1u8 => DriverHeartbeatCmdEnum::Sync,
            0u8 => DriverHeartbeatCmdEnum::Noop,
            _ => DriverHeartbeatCmdEnum::_Other(val),
        }
    }
}
impl From<DriverHeartbeatCmdEnum> for u8 {
    fn from(val: DriverHeartbeatCmdEnum) -> Self {
        match val {
            DriverHeartbeatCmdEnum::Reboot => 2u8,
            DriverHeartbeatCmdEnum::Sync => 1u8,
            DriverHeartbeatCmdEnum::Noop => 0u8,
            DriverHeartbeatCmdEnum::_Other(v) => v,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoDebugTestEnumEnum {
    Two,
    One,
    _Other(u8),
}
impl From<u8> for IoDebugTestEnumEnum {
    fn from(val: u8) -> Self {
        match val {
            2u8 => IoDebugTestEnumEnum::Two,
            1u8 => IoDebugTestEnumEnum::One,
            _ => IoDebugTestEnumEnum::_Other(val),
        }
    }
}
impl From<IoDebugTestEnumEnum> for u8 {
    fn from(val: IoDebugTestEnumEnum) -> Self {
        match val {
            IoDebugTestEnumEnum::Two => 2u8,
            IoDebugTestEnumEnum::One => 1u8,
            IoDebugTestEnumEnum::_Other(v) => v,
        }
    }
}
pub trait GeneratedCanMessage<const LEN: usize>: Sized {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError>;
    fn encode(&self) -> [u8; LEN];
}
#[derive(Debug, Clone)]
pub enum Msg {
    DriverHeartbeat(DriverHeartbeatMsg),
    IoDebug(IoDebugMsg),
    MotorCmd(MotorCmdMsg),
    MotorStatus(MotorStatusMsg),
    SensorSonars(SensorSonarsMsg),
}
impl Msg {
    fn try_from(frame: &impl Frame) -> Result<Self, CanError> {
        let result = match frame.id() {
            DriverHeartbeatMsg::ID => {
                Msg::DriverHeartbeat(DriverHeartbeatMsg::try_from_frame(frame)?)
            }
            IoDebugMsg::ID => Msg::IoDebug(IoDebugMsg::try_from_frame(frame)?),
            MotorCmdMsg::ID => Msg::MotorCmd(MotorCmdMsg::try_from_frame(frame)?),
            MotorStatusMsg::ID => {
                Msg::MotorStatus(MotorStatusMsg::try_from_frame(frame)?)
            }
            SensorSonarsMsg::ID => {
                Msg::SensorSonars(SensorSonarsMsg::try_from_frame(frame)?)
            }
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
pub struct DriverHeartbeatMsg {
    data: [u8; 1usize],
}
impl DriverHeartbeatMsg {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(100u16) });
    pub const LEN: usize = 1usize;
    pub fn new(driver_heartbeat_cmd: DriverHeartbeatCmdEnum) -> Result<Self, CanError> {
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
    pub fn driver_heartbeat_cmd(&self) -> DriverHeartbeatCmdEnum {
        let raw_driver_heartbeat_cmd = self
            .data
            .view_bits::<Lsb0>()[0usize..8usize]
            .load_le::<u8>();
        DriverHeartbeatCmdEnum::from(raw_driver_heartbeat_cmd as u8)
    }
    ///Set value of DRIVER_HEARTBEAT_cmd
    ///- Min: 0
    ///- Max: 0
    pub fn set_driver_heartbeat_cmd(
        &mut self,
        value: DriverHeartbeatCmdEnum,
    ) -> Result<(), CanError> {
        self.data.view_bits_mut::<Lsb0>()[0usize..8usize].store_le(u8::from(value));
        Ok(())
    }
}
impl GeneratedCanMessage<{ Self::LEN }> for DriverHeartbeatMsg {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        if frame.data().len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        if frame.id() != Self::ID {
            return Err(CanError::InvalidFrameId);
        }
        let mut buf = [0u8; Self::LEN];
        buf.copy_from_slice(&frame.data()[..Self::LEN]);
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
pub struct IoDebugMsg {
    data: [u8; 4usize],
}
impl IoDebugMsg {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(500u16) });
    pub const LEN: usize = 4usize;
    pub fn new(
        io_debug_test_unsigned: u8,
        io_debug_test_enum: IoDebugTestEnumEnum,
        io_debug_test_signed: i8,
        io_debug_test_float: f32,
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
        let raw_io_debug_test_unsigned = self
            .data
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
    pub fn io_debug_test_enum(&self) -> IoDebugTestEnumEnum {
        let raw_io_debug_test_enum = self
            .data
            .view_bits::<Lsb0>()[8usize..16usize]
            .load_le::<u8>();
        IoDebugTestEnumEnum::from(raw_io_debug_test_enum as u8)
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
        let raw_io_debug_test_signed = self
            .data
            .view_bits::<Lsb0>()[16usize..24usize]
            .load_le::<u8>();
        (raw_io_debug_test_signed as i8) * (1i8) + (0i8)
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
    pub fn io_debug_test_float(&self) -> f32 {
        let raw_io_debug_test_float = self
            .data
            .view_bits::<Lsb0>()[24usize..32usize]
            .load_le::<u8>();
        (raw_io_debug_test_float as f32) * (0.5f32) + (0f32)
    }
    ///Set value of IO_DEBUG_test_unsigned
    ///- Min: 0
    ///- Max: 0
    pub fn set_io_debug_test_unsigned(&mut self, value: u8) -> Result<(), CanError> {
        if value > 0u8 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[0usize..8usize]
            .store_le((value - (0u8)) / (1u8));
        Ok(())
    }
    ///Set value of IO_DEBUG_test_enum
    ///- Min: 0
    ///- Max: 0
    pub fn set_io_debug_test_enum(
        &mut self,
        value: IoDebugTestEnumEnum,
    ) -> Result<(), CanError> {
        self.data.view_bits_mut::<Lsb0>()[8usize..16usize].store_le(u8::from(value));
        Ok(())
    }
    ///Set value of IO_DEBUG_test_signed
    ///- Min: 0
    ///- Max: 0
    pub fn set_io_debug_test_signed(&mut self, value: i8) -> Result<(), CanError> {
        if value < 0i8 || value > 0i8 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[16usize..24usize]
            .store_le((value - (0i8)) / (1i8));
        Ok(())
    }
    ///Set value of IO_DEBUG_test_float
    ///- Min: 0
    ///- Max: 0
    pub fn set_io_debug_test_float(&mut self, value: f32) -> Result<(), CanError> {
        if value < 0f32 || value > 0f32 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[24usize..32usize]
            .store_le(((value - (0f32)) / (0.5f32)) as u8);
        Ok(())
    }
}
impl GeneratedCanMessage<{ Self::LEN }> for IoDebugMsg {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        if frame.data().len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        if frame.id() != Self::ID {
            return Err(CanError::InvalidFrameId);
        }
        let mut buf = [0u8; Self::LEN];
        buf.copy_from_slice(&frame.data()[..Self::LEN]);
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
pub struct MotorCmdMsg {
    data: [u8; 1usize],
}
impl MotorCmdMsg {
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
        let raw_motor_cmd_steer = self
            .data
            .view_bits::<Lsb0>()[0usize..4usize]
            .load_le::<u8>();
        (raw_motor_cmd_steer as i8) * (1i8) + (-5i8)
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
        let raw_motor_cmd_drive = self
            .data
            .view_bits::<Lsb0>()[4usize..8usize]
            .load_le::<u8>();
        (raw_motor_cmd_drive) * (1u8) + (0u8)
    }
    ///Set value of MOTOR_CMD_steer
    ///- Min: -5
    ///- Max: 5
    pub fn set_motor_cmd_steer(&mut self, value: i8) -> Result<(), CanError> {
        if value < -5i8 || value > 5i8 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[0usize..4usize]
            .store_le((value - (-5i8)) / (1i8));
        Ok(())
    }
    ///Set value of MOTOR_CMD_drive
    ///- Min: 0
    ///- Max: 9
    pub fn set_motor_cmd_drive(&mut self, value: u8) -> Result<(), CanError> {
        if value > 9u8 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[4usize..8usize]
            .store_le((value - (0u8)) / (1u8));
        Ok(())
    }
}
impl GeneratedCanMessage<{ Self::LEN }> for MotorCmdMsg {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        if frame.data().len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        if frame.id() != Self::ID {
            return Err(CanError::InvalidFrameId);
        }
        let mut buf = [0u8; Self::LEN];
        buf.copy_from_slice(&frame.data()[..Self::LEN]);
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
pub struct MotorStatusMsg {
    data: [u8; 3usize],
}
impl MotorStatusMsg {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(400u16) });
    pub const LEN: usize = 3usize;
    pub fn new(
        motor_status_wheel_error: bool,
        motor_status_speed_kph: f32,
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
    pub fn motor_status_wheel_error(&self) -> bool {
        let raw_motor_status_wheel_error = self
            .data
            .view_bits::<Lsb0>()[0usize..1usize]
            .load_le::<u8>();
        raw_motor_status_wheel_error == 1
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
    pub fn motor_status_speed_kph(&self) -> f32 {
        let raw_motor_status_speed_kph = self
            .data
            .view_bits::<Lsb0>()[8usize..24usize]
            .load_le::<u16>();
        (raw_motor_status_speed_kph as f32) * (0.001f32) + (0f32)
    }
    ///Set value of MOTOR_STATUS_wheel_error
    ///- Min: 0
    ///- Max: 0
    pub fn set_motor_status_wheel_error(&mut self, value: bool) -> Result<(), CanError> {
        self.data.view_bits_mut::<Lsb0>()[0usize..1usize].store_le(u8::from(value));
        Ok(())
    }
    ///Set value of MOTOR_STATUS_speed_kph
    ///- Min: 0
    ///- Max: 0
    pub fn set_motor_status_speed_kph(&mut self, value: f32) -> Result<(), CanError> {
        if value < 0f32 || value > 0f32 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[8usize..24usize]
            .store_le(((value - (0f32)) / (0.001f32)) as u16);
        Ok(())
    }
}
impl GeneratedCanMessage<{ Self::LEN }> for MotorStatusMsg {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        if frame.data().len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        if frame.id() != Self::ID {
            return Err(CanError::InvalidFrameId);
        }
        let mut buf = [0u8; Self::LEN];
        buf.copy_from_slice(&frame.data()[..Self::LEN]);
        Ok(Self { data: buf })
    }
    fn encode(&self) -> [u8; Self::LEN] {
        self.data
    }
}
#[derive(Debug, Clone)]
pub enum SensorSonarsMsgMux {
    V0(SensorSonarsMsgMux0),
    V1(SensorSonarsMsgMux1),
}
#[derive(Debug, Clone, Default)]
pub struct SensorSonarsMsgMux0 {
    data: [u8; 8usize],
}
impl SensorSonarsMsgMux0 {
    pub fn new(
        sensor_sonars_left: f32,
        sensor_sonars_middle: f32,
        sensor_sonars_right: f32,
        sensor_sonars_rear: f32,
    ) -> Result<Self, CanError> {
        let mut msg = Self { data: [0u8; 8usize] };
        msg.set_sensor_sonars_left(sensor_sonars_left)?;
        msg.set_sensor_sonars_middle(sensor_sonars_middle)?;
        msg.set_sensor_sonars_right(sensor_sonars_right)?;
        msg.set_sensor_sonars_rear(sensor_sonars_rear)?;
        Ok(msg)
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
    pub fn sensor_sonars_left(&self) -> f32 {
        let raw_sensor_sonars_left = self
            .data
            .view_bits::<Lsb0>()[16usize..28usize]
            .load_le::<u16>();
        (raw_sensor_sonars_left as f32) * (0.1f32) + (0f32)
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
    pub fn sensor_sonars_middle(&self) -> f32 {
        let raw_sensor_sonars_middle = self
            .data
            .view_bits::<Lsb0>()[28usize..40usize]
            .load_le::<u16>();
        (raw_sensor_sonars_middle as f32) * (0.1f32) + (0f32)
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
    pub fn sensor_sonars_right(&self) -> f32 {
        let raw_sensor_sonars_right = self
            .data
            .view_bits::<Lsb0>()[40usize..52usize]
            .load_le::<u16>();
        (raw_sensor_sonars_right as f32) * (0.1f32) + (0f32)
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
    pub fn sensor_sonars_rear(&self) -> f32 {
        let raw_sensor_sonars_rear = self
            .data
            .view_bits::<Lsb0>()[52usize..64usize]
            .load_le::<u16>();
        (raw_sensor_sonars_rear as f32) * (0.1f32) + (0f32)
    }
    ///Set value of SENSOR_SONARS_left
    ///- Min: 0
    ///- Max: 0
    pub fn set_sensor_sonars_left(&mut self, value: f32) -> Result<(), CanError> {
        if value < 0f32 || value > 0f32 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[16usize..28usize]
            .store_le(((value - (0f32)) / (0.1f32)) as u16);
        Ok(())
    }
    ///Set value of SENSOR_SONARS_middle
    ///- Min: 0
    ///- Max: 0
    pub fn set_sensor_sonars_middle(&mut self, value: f32) -> Result<(), CanError> {
        if value < 0f32 || value > 0f32 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[28usize..40usize]
            .store_le(((value - (0f32)) / (0.1f32)) as u16);
        Ok(())
    }
    ///Set value of SENSOR_SONARS_right
    ///- Min: 0
    ///- Max: 0
    pub fn set_sensor_sonars_right(&mut self, value: f32) -> Result<(), CanError> {
        if value < 0f32 || value > 0f32 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[40usize..52usize]
            .store_le(((value - (0f32)) / (0.1f32)) as u16);
        Ok(())
    }
    ///Set value of SENSOR_SONARS_rear
    ///- Min: 0
    ///- Max: 0
    pub fn set_sensor_sonars_rear(&mut self, value: f32) -> Result<(), CanError> {
        if value < 0f32 || value > 0f32 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[52usize..64usize]
            .store_le(((value - (0f32)) / (0.1f32)) as u16);
        Ok(())
    }
}
#[derive(Debug, Clone, Default)]
pub struct SensorSonarsMsgMux1 {
    data: [u8; 8usize],
}
impl SensorSonarsMsgMux1 {
    pub fn new(
        sensor_sonars_no_filt_left: f32,
        sensor_sonars_no_filt_middle: f32,
        sensor_sonars_no_filt_right: f32,
        sensor_sonars_no_filt_rear: f32,
    ) -> Result<Self, CanError> {
        let mut msg = Self { data: [0u8; 8usize] };
        msg.set_sensor_sonars_no_filt_left(sensor_sonars_no_filt_left)?;
        msg.set_sensor_sonars_no_filt_middle(sensor_sonars_no_filt_middle)?;
        msg.set_sensor_sonars_no_filt_right(sensor_sonars_no_filt_right)?;
        msg.set_sensor_sonars_no_filt_rear(sensor_sonars_no_filt_rear)?;
        Ok(msg)
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
    pub fn sensor_sonars_no_filt_left(&self) -> f32 {
        let raw_sensor_sonars_no_filt_left = self
            .data
            .view_bits::<Lsb0>()[16usize..28usize]
            .load_le::<u16>();
        (raw_sensor_sonars_no_filt_left as f32) * (0.1f32) + (0f32)
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
    pub fn sensor_sonars_no_filt_middle(&self) -> f32 {
        let raw_sensor_sonars_no_filt_middle = self
            .data
            .view_bits::<Lsb0>()[28usize..40usize]
            .load_le::<u16>();
        (raw_sensor_sonars_no_filt_middle as f32) * (0.1f32) + (0f32)
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
    pub fn sensor_sonars_no_filt_right(&self) -> f32 {
        let raw_sensor_sonars_no_filt_right = self
            .data
            .view_bits::<Lsb0>()[40usize..52usize]
            .load_le::<u16>();
        (raw_sensor_sonars_no_filt_right as f32) * (0.1f32) + (0f32)
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
    pub fn sensor_sonars_no_filt_rear(&self) -> f32 {
        let raw_sensor_sonars_no_filt_rear = self
            .data
            .view_bits::<Lsb0>()[52usize..64usize]
            .load_le::<u16>();
        (raw_sensor_sonars_no_filt_rear as f32) * (0.1f32) + (0f32)
    }
    ///Set value of SENSOR_SONARS_no_filt_left
    ///- Min: 0
    ///- Max: 0
    pub fn set_sensor_sonars_no_filt_left(
        &mut self,
        value: f32,
    ) -> Result<(), CanError> {
        if value < 0f32 || value > 0f32 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[16usize..28usize]
            .store_le(((value - (0f32)) / (0.1f32)) as u16);
        Ok(())
    }
    ///Set value of SENSOR_SONARS_no_filt_middle
    ///- Min: 0
    ///- Max: 0
    pub fn set_sensor_sonars_no_filt_middle(
        &mut self,
        value: f32,
    ) -> Result<(), CanError> {
        if value < 0f32 || value > 0f32 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[28usize..40usize]
            .store_le(((value - (0f32)) / (0.1f32)) as u16);
        Ok(())
    }
    ///Set value of SENSOR_SONARS_no_filt_right
    ///- Min: 0
    ///- Max: 0
    pub fn set_sensor_sonars_no_filt_right(
        &mut self,
        value: f32,
    ) -> Result<(), CanError> {
        if value < 0f32 || value > 0f32 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[40usize..52usize]
            .store_le(((value - (0f32)) / (0.1f32)) as u16);
        Ok(())
    }
    ///Set value of SENSOR_SONARS_no_filt_rear
    ///- Min: 0
    ///- Max: 0
    pub fn set_sensor_sonars_no_filt_rear(
        &mut self,
        value: f32,
    ) -> Result<(), CanError> {
        if value < 0f32 || value > 0f32 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[52usize..64usize]
            .store_le(((value - (0f32)) / (0.1f32)) as u16);
        Ok(())
    }
}
///SENSOR_SONARS
///- ID: Standard 200 (0xC8)
///- Size: 8 bytes
///- Transmitter: SENSOR
#[derive(Debug, Clone)]
pub struct SensorSonarsMsg {
    data: [u8; 8usize],
}
impl SensorSonarsMsg {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(200u16) });
    pub const LEN: usize = 8usize;
    pub fn new(
        sensor_sonars_err_count: u16,
        mux: SensorSonarsMsgMux,
    ) -> Result<Self, CanError> {
        let mut msg = Self { data: [0u8; Self::LEN] };
        msg.set_sensor_sonars_err_count(sensor_sonars_err_count)?;
        match mux {
            SensorSonarsMsgMux::V0(v) => {
                msg.set_mux0(v)?;
            }
            SensorSonarsMsgMux::V1(v) => {
                msg.set_mux1(v)?;
            }
        }
        Ok(msg)
    }
    pub fn mux(&self) -> Result<SensorSonarsMsgMux, CanError> {
        let raw_sensor_sonars_mux = self
            .data
            .view_bits::<Lsb0>()[0usize..4usize]
            .load_le::<u8>();
        match raw_sensor_sonars_mux {
            0 => {
                Ok(
                    SensorSonarsMsgMux::V0(SensorSonarsMsgMux0 {
                        data: self.data,
                    }),
                )
            }
            1 => {
                Ok(
                    SensorSonarsMsgMux::V1(SensorSonarsMsgMux1 {
                        data: self.data,
                    }),
                )
            }
            _ => Err(CanError::UnknownMuxValue),
        }
    }
    pub fn set_mux0(&mut self, value: SensorSonarsMsgMux0) -> Result<(), CanError> {
        let b0 = BitArray::<_, LocalBits>::new(self.data);
        let b1 = BitArray::<_, LocalBits>::new(value.data);
        self.data = b0.bitor(b1).into_inner();
        self.data.view_bits_mut::<Lsb0>()[0usize..4usize].store_le(0u64);
        Ok(())
    }
    pub fn set_mux1(&mut self, value: SensorSonarsMsgMux1) -> Result<(), CanError> {
        let b0 = BitArray::<_, LocalBits>::new(self.data);
        let b1 = BitArray::<_, LocalBits>::new(value.data);
        self.data = b0.bitor(b1).into_inner();
        self.data.view_bits_mut::<Lsb0>()[0usize..4usize].store_le(1u64);
        Ok(())
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
        let raw_sensor_sonars_err_count = self
            .data
            .view_bits::<Lsb0>()[4usize..16usize]
            .load_le::<u16>();
        (raw_sensor_sonars_err_count) * (1u16) + (0u16)
    }
    ///Set value of SENSOR_SONARS_err_count
    ///- Min: 0
    ///- Max: 0
    pub fn set_sensor_sonars_err_count(&mut self, value: u16) -> Result<(), CanError> {
        if value > 0u16 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[4usize..16usize]
            .store_le((value - (0u16)) / (1u16));
        Ok(())
    }
}
impl GeneratedCanMessage<{ Self::LEN }> for SensorSonarsMsg {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        if frame.data().len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        if frame.id() != Self::ID {
            return Err(CanError::InvalidFrameId);
        }
        let mut buf = [0u8; Self::LEN];
        buf.copy_from_slice(&frame.data()[..Self::LEN]);
        Ok(Self { data: buf })
    }
    fn encode(&self) -> [u8; Self::LEN] {
        self.data
    }
}
#[cfg(test)]
mod generated_tests {
    use super::*;
    use arbitrary::Unstructured;
    const SEEDS: &[&[u8]] = &[
        &[0u8; 128],
        &[1u8; 128],
        &[2u8; 128],
        &[3u8; 128],
        &[5u8; 128],
        &[8u8; 128],
        &[13u8; 128],
        &[21u8; 128],
        &[34u8; 128],
        &[55u8; 128],
    ];
    #[test]
    fn test_driver_heartbeat_msg() {
        for seed in SEEDS {
            let mut u = Unstructured::new(seed);
            let driver_heartbeat_cmd_value = DriverHeartbeatCmdEnum::Reboot;
            let mut msg = DriverHeartbeatMsg::new(driver_heartbeat_cmd_value)
                .expect("constructor should accept generated test values");
            assert_eq!(
                msg.driver_heartbeat_cmd(), driver_heartbeat_cmd_value,
                "getter `{}` returned unexpected value",
                stringify!(driver_heartbeat_cmd),
            );
            let driver_heartbeat_cmd_next_value = DriverHeartbeatCmdEnum::Reboot;
            msg.set_driver_heartbeat_cmd(driver_heartbeat_cmd_next_value)
                .expect("setter should accept generated test value");
            assert_eq!(
                msg.driver_heartbeat_cmd(), driver_heartbeat_cmd_next_value,
                "getter `{}` returned unexpected value",
                stringify!(driver_heartbeat_cmd),
            );
        }
    }
    #[test]
    fn test_io_debug_msg() {
        for seed in SEEDS {
            let mut u = Unstructured::new(seed);
            let io_debug_test_unsigned_value: u8 = {
                let raw = u
                    .int_in_range(0u8..=0u8)
                    .expect("failed to generate physical interger value");
                raw
            };
            let io_debug_test_enum_value = IoDebugTestEnumEnum::Two;
            let io_debug_test_signed_value: i8 = {
                let raw = u
                    .int_in_range(0i8..=0i8)
                    .expect("failed to generate physical interger value");
                raw
            };
            let io_debug_test_float_value: f32 = {
                let mut x: f32 = u
                    .arbitrary()
                    .expect("failed to generate physical float value");
                x = x.min(0f32);
                x.max(0f32)
            };
            let mut msg = IoDebugMsg::new(
                    io_debug_test_unsigned_value,
                    io_debug_test_enum_value,
                    io_debug_test_signed_value,
                    io_debug_test_float_value,
                )
                .expect("constructor should accept generated test values");
            assert_eq!(
                msg.io_debug_test_unsigned(), io_debug_test_unsigned_value,
                "getter `{}` returned unexpected value",
                stringify!(io_debug_test_unsigned),
            );
            assert_eq!(
                msg.io_debug_test_enum(), io_debug_test_enum_value,
                "getter `{}` returned unexpected value", stringify!(io_debug_test_enum),
            );
            assert_eq!(
                msg.io_debug_test_signed(), io_debug_test_signed_value,
                "getter `{}` returned unexpected value",
                stringify!(io_debug_test_signed),
            );
            let actual = msg.io_debug_test_float();
            let expected = io_debug_test_float_value;
            let tolerance = (expected.abs() * 0.0001).max(0.0001);
            assert!(
                (actual - expected).abs() <= tolerance,
                "getter `{}` returned {:?}, expected {:?}",
                stringify!(io_debug_test_float), actual, expected,
            );
            let io_debug_test_unsigned_next_value: u8 = {
                let raw = u
                    .int_in_range(0u8..=0u8)
                    .expect("failed to generate physical interger value");
                raw
            };
            let io_debug_test_enum_next_value = IoDebugTestEnumEnum::Two;
            let io_debug_test_signed_next_value: i8 = {
                let raw = u
                    .int_in_range(0i8..=0i8)
                    .expect("failed to generate physical interger value");
                raw
            };
            let io_debug_test_float_next_value: f32 = {
                let mut x: f32 = u
                    .arbitrary()
                    .expect("failed to generate physical float value");
                x = x.min(0f32);
                x.max(0f32)
            };
            msg.set_io_debug_test_unsigned(io_debug_test_unsigned_next_value)
                .expect("setter should accept generated test value");
            msg.set_io_debug_test_enum(io_debug_test_enum_next_value)
                .expect("setter should accept generated test value");
            msg.set_io_debug_test_signed(io_debug_test_signed_next_value)
                .expect("setter should accept generated test value");
            msg.set_io_debug_test_float(io_debug_test_float_next_value)
                .expect("setter should accept generated test value");
            assert_eq!(
                msg.io_debug_test_unsigned(), io_debug_test_unsigned_next_value,
                "getter `{}` returned unexpected value",
                stringify!(io_debug_test_unsigned),
            );
            assert_eq!(
                msg.io_debug_test_enum(), io_debug_test_enum_next_value,
                "getter `{}` returned unexpected value", stringify!(io_debug_test_enum),
            );
            assert_eq!(
                msg.io_debug_test_signed(), io_debug_test_signed_next_value,
                "getter `{}` returned unexpected value",
                stringify!(io_debug_test_signed),
            );
            let actual = msg.io_debug_test_float();
            let expected = io_debug_test_float_next_value;
            let tolerance = (expected.abs() * 0.0001).max(0.0001);
            assert!(
                (actual - expected).abs() <= tolerance,
                "getter `{}` returned {:?}, expected {:?}",
                stringify!(io_debug_test_float), actual, expected,
            );
        }
    }
    #[test]
    fn test_motor_cmd_msg() {
        for seed in SEEDS {
            let mut u = Unstructured::new(seed);
            let motor_cmd_steer_value: i8 = {
                let raw = u
                    .int_in_range(-5i8..=5i8)
                    .expect("failed to generate physical interger value");
                raw
            };
            let motor_cmd_drive_value: u8 = {
                let raw = u
                    .int_in_range(0u8..=9u8)
                    .expect("failed to generate physical interger value");
                raw
            };
            let mut msg = MotorCmdMsg::new(motor_cmd_steer_value, motor_cmd_drive_value)
                .expect("constructor should accept generated test values");
            assert_eq!(
                msg.motor_cmd_steer(), motor_cmd_steer_value,
                "getter `{}` returned unexpected value", stringify!(motor_cmd_steer),
            );
            assert_eq!(
                msg.motor_cmd_drive(), motor_cmd_drive_value,
                "getter `{}` returned unexpected value", stringify!(motor_cmd_drive),
            );
            let motor_cmd_steer_next_value: i8 = {
                let raw = u
                    .int_in_range(-5i8..=5i8)
                    .expect("failed to generate physical interger value");
                raw
            };
            let motor_cmd_drive_next_value: u8 = {
                let raw = u
                    .int_in_range(0u8..=9u8)
                    .expect("failed to generate physical interger value");
                raw
            };
            msg.set_motor_cmd_steer(motor_cmd_steer_next_value)
                .expect("setter should accept generated test value");
            msg.set_motor_cmd_drive(motor_cmd_drive_next_value)
                .expect("setter should accept generated test value");
            assert_eq!(
                msg.motor_cmd_steer(), motor_cmd_steer_next_value,
                "getter `{}` returned unexpected value", stringify!(motor_cmd_steer),
            );
            assert_eq!(
                msg.motor_cmd_drive(), motor_cmd_drive_next_value,
                "getter `{}` returned unexpected value", stringify!(motor_cmd_drive),
            );
        }
    }
    #[test]
    fn test_motor_status_msg() {
        for seed in SEEDS {
            let mut u = Unstructured::new(seed);
            let motor_status_wheel_error_value: bool = u
                .arbitrary()
                .expect("failed to generate bool");
            let motor_status_speed_kph_value: f32 = {
                let mut x: f32 = u
                    .arbitrary()
                    .expect("failed to generate physical float value");
                x = x.min(0f32);
                x.max(0f32)
            };
            let mut msg = MotorStatusMsg::new(
                    motor_status_wheel_error_value,
                    motor_status_speed_kph_value,
                )
                .expect("constructor should accept generated test values");
            assert_eq!(
                msg.motor_status_wheel_error(), motor_status_wheel_error_value,
                "getter `{}` returned unexpected value",
                stringify!(motor_status_wheel_error),
            );
            let actual = msg.motor_status_speed_kph();
            let expected = motor_status_speed_kph_value;
            let tolerance = (expected.abs() * 0.0001).max(0.0001);
            assert!(
                (actual - expected).abs() <= tolerance,
                "getter `{}` returned {:?}, expected {:?}",
                stringify!(motor_status_speed_kph), actual, expected,
            );
            let motor_status_wheel_error_next_value: bool = u
                .arbitrary()
                .expect("failed to generate bool");
            let motor_status_speed_kph_next_value: f32 = {
                let mut x: f32 = u
                    .arbitrary()
                    .expect("failed to generate physical float value");
                x = x.min(0f32);
                x.max(0f32)
            };
            msg.set_motor_status_wheel_error(motor_status_wheel_error_next_value)
                .expect("setter should accept generated test value");
            msg.set_motor_status_speed_kph(motor_status_speed_kph_next_value)
                .expect("setter should accept generated test value");
            assert_eq!(
                msg.motor_status_wheel_error(), motor_status_wheel_error_next_value,
                "getter `{}` returned unexpected value",
                stringify!(motor_status_wheel_error),
            );
            let actual = msg.motor_status_speed_kph();
            let expected = motor_status_speed_kph_next_value;
            let tolerance = (expected.abs() * 0.0001).max(0.0001);
            assert!(
                (actual - expected).abs() <= tolerance,
                "getter `{}` returned {:?}, expected {:?}",
                stringify!(motor_status_speed_kph), actual, expected,
            );
        }
    }
}
