use embedded_can::{Frame, Id, StandardId, ExtendedId};
use bitvec::prelude::*;
#[derive(Debug, Clone)]
pub enum CanError {
    UnknownFrameId,
    UnknownMuxValue,
    InvalidPayloadSize,
    ValueOutOfRange,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverHeartbeatCmd {
    Reboot,
    Sync,
    Noop,
}
impl From<u8> for DriverHeartbeatCmd {
    fn from(val: u8) -> Self {
        match val {
            2u8 => DriverHeartbeatCmd::Reboot,
            1u8 => DriverHeartbeatCmd::Sync,
            0u8 => DriverHeartbeatCmd::Noop,
            _ => panic!("Invalid enum value"),
        }
    }
}
impl From<DriverHeartbeatCmd> for u8 {
    fn from(val: DriverHeartbeatCmd) -> Self {
        match val {
            DriverHeartbeatCmd::Reboot => 2u8,
            DriverHeartbeatCmd::Sync => 1u8,
            DriverHeartbeatCmd::Noop => 0u8,
            _ => panic!("Invalid enum value"),
        }
    }
}
#[derive(Debug, Clone)]
pub struct DriverHeartbeat {
    pub driver_heartbeat_cmd: DriverHeartbeatCmd,
}
impl DriverHeartbeat {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(100u16) });
    pub const LEN: usize = 1usize;
    pub fn new(driver_heartbeat_cmd: DriverHeartbeatCmd) -> Result<Self, CanError> {
        let mut msg = Self { driver_heartbeat_cmd };
        msg.set_driver_heartbeat_cmd(msg.driver_heartbeat_cmd)?;
        Ok(msg)
    }
    pub fn driver_heartbeat_cmd(&self) -> DriverHeartbeatCmd {
        self.driver_heartbeat_cmd
    }
    pub fn set_driver_heartbeat_cmd(
        &mut self,
        value: DriverHeartbeatCmd,
    ) -> Result<(), CanError> {
        self.driver_heartbeat_cmd = value;
        Ok(())
    }
}
impl CanMessage<{ DriverHeartbeat::LEN }> for DriverHeartbeat {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        if data.len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        let raw_driver_heartbeat_cmd = data
            .view_bits::<Lsb0>()[0usize..8usize]
            .load_le::<u8>();
        Ok(Self {
            driver_heartbeat_cmd: DriverHeartbeatCmd::from(
                raw_driver_heartbeat_cmd as u8,
            ),
        })
    }
    fn encode(&self) -> [u8; DriverHeartbeat::LEN] {
        let mut data = [0u8; DriverHeartbeat::LEN];
        data.view_bits_mut::<Lsb0>()[0usize..8usize]
            .store_le(u8::from(self.driver_heartbeat_cmd));
        data
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoDebugTestEnum {
    Two,
    One,
}
impl From<u8> for IoDebugTestEnum {
    fn from(val: u8) -> Self {
        match val {
            2u8 => IoDebugTestEnum::Two,
            1u8 => IoDebugTestEnum::One,
            _ => panic!("Invalid enum value"),
        }
    }
}
impl From<IoDebugTestEnum> for u8 {
    fn from(val: IoDebugTestEnum) -> Self {
        match val {
            IoDebugTestEnum::Two => 2u8,
            IoDebugTestEnum::One => 1u8,
            _ => panic!("Invalid enum value"),
        }
    }
}
#[derive(Debug, Clone)]
pub struct IoDebug {
    pub io_debug_test_unsigned: u8,
    pub io_debug_test_enum: IoDebugTestEnum,
    pub io_debug_test_signed: i8,
    pub io_debug_test_float: f64,
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
        let mut msg = Self {
            io_debug_test_unsigned,
            io_debug_test_enum,
            io_debug_test_signed,
            io_debug_test_float,
        };
        msg.set_io_debug_test_unsigned(msg.io_debug_test_unsigned)?;
        msg.set_io_debug_test_enum(msg.io_debug_test_enum)?;
        msg.set_io_debug_test_signed(msg.io_debug_test_signed)?;
        msg.set_io_debug_test_float(msg.io_debug_test_float)?;
        Ok(msg)
    }
    pub fn io_debug_test_unsigned(&self) -> u8 {
        self.io_debug_test_unsigned
    }
    pub fn io_debug_test_enum(&self) -> IoDebugTestEnum {
        self.io_debug_test_enum
    }
    pub fn io_debug_test_signed(&self) -> i8 {
        self.io_debug_test_signed
    }
    pub fn io_debug_test_float(&self) -> f64 {
        self.io_debug_test_float
    }
    pub fn set_io_debug_test_unsigned(&mut self, value: u8) -> Result<(), CanError> {
        if value < 0u8 || value > 0u8 {
            return Err(CanError::ValueOutOfRange);
        }
        self.io_debug_test_unsigned = value;
        Ok(())
    }
    pub fn set_io_debug_test_enum(
        &mut self,
        value: IoDebugTestEnum,
    ) -> Result<(), CanError> {
        self.io_debug_test_enum = value;
        Ok(())
    }
    pub fn set_io_debug_test_signed(&mut self, value: i8) -> Result<(), CanError> {
        if value < 0i8 || value > 0i8 {
            return Err(CanError::ValueOutOfRange);
        }
        self.io_debug_test_signed = value;
        Ok(())
    }
    pub fn set_io_debug_test_float(&mut self, value: f64) -> Result<(), CanError> {
        if value < 0f64 || value > 0f64 {
            return Err(CanError::ValueOutOfRange);
        }
        self.io_debug_test_float = value;
        Ok(())
    }
}
impl CanMessage<{ IoDebug::LEN }> for IoDebug {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        if data.len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        let raw_io_debug_test_unsigned = data
            .view_bits::<Lsb0>()[0usize..8usize]
            .load_le::<u8>();
        let raw_io_debug_test_enum = data
            .view_bits::<Lsb0>()[8usize..16usize]
            .load_le::<u8>();
        let raw_io_debug_test_signed = data
            .view_bits::<Lsb0>()[16usize..24usize]
            .load_le::<i8>();
        let raw_io_debug_test_float = data
            .view_bits::<Lsb0>()[24usize..32usize]
            .load_le::<u8>();
        Ok(Self {
            io_debug_test_unsigned: (raw_io_debug_test_unsigned) * (1u8) + (0u8),
            io_debug_test_enum: IoDebugTestEnum::from(raw_io_debug_test_enum as u8),
            io_debug_test_signed: (raw_io_debug_test_signed) * (1i8) + (0i8),
            io_debug_test_float: (raw_io_debug_test_float as f64) * (0.5f64) + (0f64),
        })
    }
    fn encode(&self) -> [u8; IoDebug::LEN] {
        let mut data = [0u8; IoDebug::LEN];
        data.view_bits_mut::<Lsb0>()[0usize..8usize]
            .store_le((self.io_debug_test_unsigned - (0u8)) / (1u8));
        data.view_bits_mut::<Lsb0>()[8usize..16usize]
            .store_le(u8::from(self.io_debug_test_enum));
        data.view_bits_mut::<Lsb0>()[16usize..24usize]
            .store_le((self.io_debug_test_signed - (0i8)) / (1i8));
        data.view_bits_mut::<Lsb0>()[24usize..32usize]
            .store_le(((self.io_debug_test_float - (0f64)) / (0.5f64)) as u8);
        data
    }
}
#[derive(Debug, Clone)]
pub struct MotorCmd {
    pub motor_cmd_steer: i8,
    pub motor_cmd_drive: u8,
}
impl MotorCmd {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(101u16) });
    pub const LEN: usize = 1usize;
    pub fn new(motor_cmd_steer: i8, motor_cmd_drive: u8) -> Result<Self, CanError> {
        let mut msg = Self {
            motor_cmd_steer,
            motor_cmd_drive,
        };
        msg.set_motor_cmd_steer(msg.motor_cmd_steer)?;
        msg.set_motor_cmd_drive(msg.motor_cmd_drive)?;
        Ok(msg)
    }
    pub fn motor_cmd_steer(&self) -> i8 {
        self.motor_cmd_steer
    }
    pub fn motor_cmd_drive(&self) -> u8 {
        self.motor_cmd_drive
    }
    pub fn set_motor_cmd_steer(&mut self, value: i8) -> Result<(), CanError> {
        if value < -5i8 || value > 5i8 {
            return Err(CanError::ValueOutOfRange);
        }
        self.motor_cmd_steer = value;
        Ok(())
    }
    pub fn set_motor_cmd_drive(&mut self, value: u8) -> Result<(), CanError> {
        if value < 0u8 || value > 9u8 {
            return Err(CanError::ValueOutOfRange);
        }
        self.motor_cmd_drive = value;
        Ok(())
    }
}
impl CanMessage<{ MotorCmd::LEN }> for MotorCmd {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        if data.len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        let raw_motor_cmd_steer = data
            .view_bits::<Lsb0>()[0usize..4usize]
            .load_le::<i8>();
        let raw_motor_cmd_drive = data
            .view_bits::<Lsb0>()[4usize..8usize]
            .load_le::<u8>();
        Ok(Self {
            motor_cmd_steer: (raw_motor_cmd_steer) * (1i8) + (-5i8),
            motor_cmd_drive: (raw_motor_cmd_drive) * (1u8) + (0u8),
        })
    }
    fn encode(&self) -> [u8; MotorCmd::LEN] {
        let mut data = [0u8; MotorCmd::LEN];
        data.view_bits_mut::<Lsb0>()[0usize..4usize]
            .store_le((self.motor_cmd_steer - (-5i8)) / (1i8));
        data.view_bits_mut::<Lsb0>()[4usize..8usize]
            .store_le((self.motor_cmd_drive - (0u8)) / (1u8));
        data
    }
}
#[derive(Debug, Clone)]
pub struct MotorStatus {
    pub motor_status_wheel_error: u8,
    pub motor_status_speed_kph: f64,
}
impl MotorStatus {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(400u16) });
    pub const LEN: usize = 3usize;
    pub fn new(
        motor_status_wheel_error: u8,
        motor_status_speed_kph: f64,
    ) -> Result<Self, CanError> {
        let mut msg = Self {
            motor_status_wheel_error,
            motor_status_speed_kph,
        };
        msg.set_motor_status_wheel_error(msg.motor_status_wheel_error)?;
        msg.set_motor_status_speed_kph(msg.motor_status_speed_kph)?;
        Ok(msg)
    }
    pub fn motor_status_wheel_error(&self) -> u8 {
        self.motor_status_wheel_error
    }
    pub fn motor_status_speed_kph(&self) -> f64 {
        self.motor_status_speed_kph
    }
    pub fn set_motor_status_wheel_error(&mut self, value: u8) -> Result<(), CanError> {
        if value < 0u8 || value > 0u8 {
            return Err(CanError::ValueOutOfRange);
        }
        self.motor_status_wheel_error = value;
        Ok(())
    }
    pub fn set_motor_status_speed_kph(&mut self, value: f64) -> Result<(), CanError> {
        if value < 0f64 || value > 0f64 {
            return Err(CanError::ValueOutOfRange);
        }
        self.motor_status_speed_kph = value;
        Ok(())
    }
}
impl CanMessage<{ MotorStatus::LEN }> for MotorStatus {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        if data.len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        let raw_motor_status_wheel_error = data
            .view_bits::<Lsb0>()[0usize..1usize]
            .load_le::<u8>();
        let raw_motor_status_speed_kph = data
            .view_bits::<Lsb0>()[8usize..24usize]
            .load_le::<u16>();
        Ok(Self {
            motor_status_wheel_error: (raw_motor_status_wheel_error) * (1u8) + (0u8),
            motor_status_speed_kph: (raw_motor_status_speed_kph as f64) * (0.001f64)
                + (0f64),
        })
    }
    fn encode(&self) -> [u8; MotorStatus::LEN] {
        let mut data = [0u8; MotorStatus::LEN];
        data.view_bits_mut::<Lsb0>()[0usize..1usize]
            .store_le((self.motor_status_wheel_error - (0u8)) / (1u8));
        data.view_bits_mut::<Lsb0>()[8usize..24usize]
            .store_le(((self.motor_status_speed_kph - (0f64)) / (0.001f64)) as u16);
        data
    }
}
#[derive(Debug, Clone)]
pub struct SensorSonarsMux0 {
    pub sensor_sonars_left: f64,
    pub sensor_sonars_middle: f64,
    pub sensor_sonars_right: f64,
    pub sensor_sonars_rear: f64,
}
impl SensorSonarsMux0 {
    pub fn new(
        sensor_sonars_left: f64,
        sensor_sonars_middle: f64,
        sensor_sonars_right: f64,
        sensor_sonars_rear: f64,
    ) -> Result<Self, CanError> {
        let mut msg = Self {
            sensor_sonars_left,
            sensor_sonars_middle,
            sensor_sonars_right,
            sensor_sonars_rear,
        };
        msg.set_sensor_sonars_left(msg.sensor_sonars_left)?;
        msg.set_sensor_sonars_middle(msg.sensor_sonars_middle)?;
        msg.set_sensor_sonars_right(msg.sensor_sonars_right)?;
        msg.set_sensor_sonars_rear(msg.sensor_sonars_rear)?;
        Ok(msg)
    }
    pub fn sensor_sonars_left(&self) -> f64 {
        self.sensor_sonars_left
    }
    pub fn sensor_sonars_middle(&self) -> f64 {
        self.sensor_sonars_middle
    }
    pub fn sensor_sonars_right(&self) -> f64 {
        self.sensor_sonars_right
    }
    pub fn sensor_sonars_rear(&self) -> f64 {
        self.sensor_sonars_rear
    }
    pub fn set_sensor_sonars_left(&mut self, value: f64) -> Result<(), CanError> {
        if value < 0f64 || value > 0f64 {
            return Err(CanError::ValueOutOfRange);
        }
        self.sensor_sonars_left = value;
        Ok(())
    }
    pub fn set_sensor_sonars_middle(&mut self, value: f64) -> Result<(), CanError> {
        if value < 0f64 || value > 0f64 {
            return Err(CanError::ValueOutOfRange);
        }
        self.sensor_sonars_middle = value;
        Ok(())
    }
    pub fn set_sensor_sonars_right(&mut self, value: f64) -> Result<(), CanError> {
        if value < 0f64 || value > 0f64 {
            return Err(CanError::ValueOutOfRange);
        }
        self.sensor_sonars_right = value;
        Ok(())
    }
    pub fn set_sensor_sonars_rear(&mut self, value: f64) -> Result<(), CanError> {
        if value < 0f64 || value > 0f64 {
            return Err(CanError::ValueOutOfRange);
        }
        self.sensor_sonars_rear = value;
        Ok(())
    }
    fn decode_from(data: &[u8]) -> Result<Self, CanError> {
        let raw_sensor_sonars_left = data
            .view_bits::<Lsb0>()[16usize..28usize]
            .load_le::<u16>();
        let raw_sensor_sonars_middle = data
            .view_bits::<Lsb0>()[28usize..40usize]
            .load_le::<u16>();
        let raw_sensor_sonars_right = data
            .view_bits::<Lsb0>()[40usize..52usize]
            .load_le::<u16>();
        let raw_sensor_sonars_rear = data
            .view_bits::<Lsb0>()[52usize..64usize]
            .load_le::<u16>();
        Ok(Self {
            sensor_sonars_left: (raw_sensor_sonars_left as f64) * (0.1f64) + (0f64),
            sensor_sonars_middle: (raw_sensor_sonars_middle as f64) * (0.1f64) + (0f64),
            sensor_sonars_right: (raw_sensor_sonars_right as f64) * (0.1f64) + (0f64),
            sensor_sonars_rear: (raw_sensor_sonars_rear as f64) * (0.1f64) + (0f64),
        })
    }
    fn encode_into(&self, data: &mut [u8]) {
        data.view_bits_mut::<Lsb0>()[16usize..28usize]
            .store_le(((self.sensor_sonars_left - (0f64)) / (0.1f64)) as u16);
        data.view_bits_mut::<Lsb0>()[28usize..40usize]
            .store_le(((self.sensor_sonars_middle - (0f64)) / (0.1f64)) as u16);
        data.view_bits_mut::<Lsb0>()[40usize..52usize]
            .store_le(((self.sensor_sonars_right - (0f64)) / (0.1f64)) as u16);
        data.view_bits_mut::<Lsb0>()[52usize..64usize]
            .store_le(((self.sensor_sonars_rear - (0f64)) / (0.1f64)) as u16);
    }
}
#[derive(Debug, Clone)]
pub struct SensorSonarsMux1 {
    pub sensor_sonars_no_filt_left: f64,
    pub sensor_sonars_no_filt_middle: f64,
    pub sensor_sonars_no_filt_right: f64,
    pub sensor_sonars_no_filt_rear: f64,
}
impl SensorSonarsMux1 {
    pub fn new(
        sensor_sonars_no_filt_left: f64,
        sensor_sonars_no_filt_middle: f64,
        sensor_sonars_no_filt_right: f64,
        sensor_sonars_no_filt_rear: f64,
    ) -> Result<Self, CanError> {
        let mut msg = Self {
            sensor_sonars_no_filt_left,
            sensor_sonars_no_filt_middle,
            sensor_sonars_no_filt_right,
            sensor_sonars_no_filt_rear,
        };
        msg.set_sensor_sonars_no_filt_left(msg.sensor_sonars_no_filt_left)?;
        msg.set_sensor_sonars_no_filt_middle(msg.sensor_sonars_no_filt_middle)?;
        msg.set_sensor_sonars_no_filt_right(msg.sensor_sonars_no_filt_right)?;
        msg.set_sensor_sonars_no_filt_rear(msg.sensor_sonars_no_filt_rear)?;
        Ok(msg)
    }
    pub fn sensor_sonars_no_filt_left(&self) -> f64 {
        self.sensor_sonars_no_filt_left
    }
    pub fn sensor_sonars_no_filt_middle(&self) -> f64 {
        self.sensor_sonars_no_filt_middle
    }
    pub fn sensor_sonars_no_filt_right(&self) -> f64 {
        self.sensor_sonars_no_filt_right
    }
    pub fn sensor_sonars_no_filt_rear(&self) -> f64 {
        self.sensor_sonars_no_filt_rear
    }
    pub fn set_sensor_sonars_no_filt_left(
        &mut self,
        value: f64,
    ) -> Result<(), CanError> {
        if value < 0f64 || value > 0f64 {
            return Err(CanError::ValueOutOfRange);
        }
        self.sensor_sonars_no_filt_left = value;
        Ok(())
    }
    pub fn set_sensor_sonars_no_filt_middle(
        &mut self,
        value: f64,
    ) -> Result<(), CanError> {
        if value < 0f64 || value > 0f64 {
            return Err(CanError::ValueOutOfRange);
        }
        self.sensor_sonars_no_filt_middle = value;
        Ok(())
    }
    pub fn set_sensor_sonars_no_filt_right(
        &mut self,
        value: f64,
    ) -> Result<(), CanError> {
        if value < 0f64 || value > 0f64 {
            return Err(CanError::ValueOutOfRange);
        }
        self.sensor_sonars_no_filt_right = value;
        Ok(())
    }
    pub fn set_sensor_sonars_no_filt_rear(
        &mut self,
        value: f64,
    ) -> Result<(), CanError> {
        if value < 0f64 || value > 0f64 {
            return Err(CanError::ValueOutOfRange);
        }
        self.sensor_sonars_no_filt_rear = value;
        Ok(())
    }
    fn decode_from(data: &[u8]) -> Result<Self, CanError> {
        let raw_sensor_sonars_no_filt_left = data
            .view_bits::<Lsb0>()[16usize..28usize]
            .load_le::<u16>();
        let raw_sensor_sonars_no_filt_middle = data
            .view_bits::<Lsb0>()[28usize..40usize]
            .load_le::<u16>();
        let raw_sensor_sonars_no_filt_right = data
            .view_bits::<Lsb0>()[40usize..52usize]
            .load_le::<u16>();
        let raw_sensor_sonars_no_filt_rear = data
            .view_bits::<Lsb0>()[52usize..64usize]
            .load_le::<u16>();
        Ok(Self {
            sensor_sonars_no_filt_left: (raw_sensor_sonars_no_filt_left as f64)
                * (0.1f64) + (0f64),
            sensor_sonars_no_filt_middle: (raw_sensor_sonars_no_filt_middle as f64)
                * (0.1f64) + (0f64),
            sensor_sonars_no_filt_right: (raw_sensor_sonars_no_filt_right as f64)
                * (0.1f64) + (0f64),
            sensor_sonars_no_filt_rear: (raw_sensor_sonars_no_filt_rear as f64)
                * (0.1f64) + (0f64),
        })
    }
    fn encode_into(&self, data: &mut [u8]) {
        data.view_bits_mut::<Lsb0>()[16usize..28usize]
            .store_le(((self.sensor_sonars_no_filt_left - (0f64)) / (0.1f64)) as u16);
        data.view_bits_mut::<Lsb0>()[28usize..40usize]
            .store_le(((self.sensor_sonars_no_filt_middle - (0f64)) / (0.1f64)) as u16);
        data.view_bits_mut::<Lsb0>()[40usize..52usize]
            .store_le(((self.sensor_sonars_no_filt_right - (0f64)) / (0.1f64)) as u16);
        data.view_bits_mut::<Lsb0>()[52usize..64usize]
            .store_le(((self.sensor_sonars_no_filt_rear - (0f64)) / (0.1f64)) as u16);
    }
}
#[derive(Debug, Clone)]
pub enum SensorSonarsMux {
    V0(SensorSonarsMux0),
    V1(SensorSonarsMux1),
}
#[derive(Debug, Clone)]
pub struct SensorSonars {
    pub sensor_sonars_err_count: u16,
    pub mux: SensorSonarsMux,
}
impl SensorSonars {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(200u16) });
    pub const LEN: usize = 8usize;
    pub fn new(
        sensor_sonars_err_count: u16,
        mux: SensorSonarsMux,
    ) -> Result<Self, CanError> {
        let mut msg = Self {
            sensor_sonars_err_count,
            mux,
        };
        msg.set_sensor_sonars_err_count(msg.sensor_sonars_err_count)?;
        Ok(msg)
    }
    pub fn sensor_sonars_err_count(&self) -> u16 {
        self.sensor_sonars_err_count
    }
    pub fn set_sensor_sonars_err_count(&mut self, value: u16) -> Result<(), CanError> {
        if value < 0u16 || value > 0u16 {
            return Err(CanError::ValueOutOfRange);
        }
        self.sensor_sonars_err_count = value;
        Ok(())
    }
}
impl CanMessage<{ SensorSonars::LEN }> for SensorSonars {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        if data.len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        let raw_sensor_sonars_err_count = data
            .view_bits::<Lsb0>()[4usize..16usize]
            .load_le::<u16>();
        let raw_sensor_sonars_mux = data
            .view_bits::<Lsb0>()[0usize..4usize]
            .load_le::<u8>();
        let mux = match raw_sensor_sonars_mux {
            0u8 => SensorSonarsMux::V0(SensorSonarsMux0::decode_from(data)?),
            1u8 => SensorSonarsMux::V1(SensorSonarsMux1::decode_from(data)?),
            _ => return Err(CanError::UnknownMuxValue),
        };
        Ok(Self {
            sensor_sonars_err_count: (raw_sensor_sonars_err_count) * (1u16) + (0u16),
            mux,
        })
    }
    fn encode(&self) -> [u8; SensorSonars::LEN] {
        let mut data = [0u8; SensorSonars::LEN];
        data.view_bits_mut::<Lsb0>()[4usize..16usize]
            .store_le((self.sensor_sonars_err_count - (0u16)) / (1u16));
        match &self.mux {
            SensorSonarsMux::V0(inner) => {
                data.view_bits_mut::<Lsb0>()[0usize..4usize].store_le(0u64 as u8);
                inner.encode_into(&mut data);
            }
            SensorSonarsMux::V1(inner) => {
                data.view_bits_mut::<Lsb0>()[0usize..4usize].store_le(1u64 as u8);
                inner.encode_into(&mut data);
            }
        }
        data
    }
}
