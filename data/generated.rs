use embedded_can::{Frame, Id, StandardId, ExtendedId};
#[derive(Debug, Clone)]
pub enum CanError {
    Err1,
    Err2,
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
            _ => return Err(CanError::Err1),
        };
        Ok(result)
    }
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
            2u8 => Self::Reboot,
            1u8 => Self::Sync,
            0u8 => Self::Noop,
            _ => Self::_Other(val),
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
        let raw_driver_heartbeat_cmd = data[0usize];
        Ok(Self {
            driver_heartbeat_cmd: DriverHeartbeatCmd::from(
                raw_driver_heartbeat_cmd as u8,
            ),
        })
    }
    fn encode(&self) -> [u8; DriverHeartbeat::LEN] {
        let mut data = [0u8; DriverHeartbeat::LEN];
        data[0usize] = u8::from(self.driver_heartbeat_cmd);
        data
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoDebugTestEnum {
    IoDebugTest2EnumTwo,
    IoDebugTest2EnumOne,
    _Other(u8),
}
impl From<u8> for IoDebugTestEnum {
    fn from(val: u8) -> Self {
        match val {
            2u8 => Self::IoDebugTest2EnumTwo,
            1u8 => Self::IoDebugTest2EnumOne,
            _ => Self::_Other(val),
        }
    }
}
impl From<IoDebugTestEnum> for u8 {
    fn from(val: IoDebugTestEnum) -> Self {
        match val {
            IoDebugTestEnum::IoDebugTest2EnumTwo => 2u8,
            IoDebugTestEnum::IoDebugTest2EnumOne => 1u8,
            IoDebugTestEnum::_Other(v) => v,
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
        let raw_io_debug_test_unsigned = data[0usize];
        let raw_io_debug_test_enum = data[1usize];
        let raw_io_debug_test_signed = data[2usize];
        let raw_io_debug_test_float = data[3usize];
        Ok(Self {
            io_debug_test_unsigned: raw_io_debug_test_unsigned as f64 * 1f64,
            io_debug_test_enum: IoDebugTestEnum::from(raw_io_debug_test_enum as u8),
            io_debug_test_signed: raw_io_debug_test_signed as f64 * 1f64,
            io_debug_test_float: raw_io_debug_test_float as f64 * 0.5f64,
        })
    }
    fn encode(&self) -> [u8; IoDebug::LEN] {
        let mut data = [0u8; IoDebug::LEN];
        data[0usize] = (self.io_debug_test_unsigned / 1f64) as u8;
        data[1usize] = u8::from(self.io_debug_test_enum);
        data[2usize] = (self.io_debug_test_signed / 1f64) as u8;
        data[3usize] = (self.io_debug_test_float / 0.5f64) as u8;
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
        let raw_motor_cmd_steer = data[0usize];
        let raw_motor_cmd_drive = data[0usize];
        Ok(Self {
            motor_cmd_steer: raw_motor_cmd_steer as f64 * 1f64,
            motor_cmd_drive: raw_motor_cmd_drive as f64 * 1f64,
        })
    }
    fn encode(&self) -> [u8; MotorCmd::LEN] {
        let mut data = [0u8; MotorCmd::LEN];
        data[0usize] = (self.motor_cmd_steer / 1f64) as u8;
        data[0usize] = (self.motor_cmd_drive / 1f64) as u8;
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
        let raw_motor_status_wheel_error = data[0usize];
        let raw_motor_status_speed_kph = u16::from_le_bytes([
            data[1usize],
            data[2usize],
        ]);
        Ok(Self {
            motor_status_wheel_error: raw_motor_status_wheel_error as f64 * 1f64,
            motor_status_speed_kph: raw_motor_status_speed_kph as f64 * 0.001f64,
        })
    }
    fn encode(&self) -> [u8; MotorStatus::LEN] {
        let mut data = [0u8; MotorStatus::LEN];
        data[0usize] = (self.motor_status_wheel_error / 1f64) as u8;
        let bytes = ((self.motor_status_speed_kph / 0.001f64) as u16).to_le_bytes();
        data[1usize] = bytes[0usize];
        data[2usize] = bytes[1usize];
        data
    }
}
#[derive(Debug, Clone)]
pub struct SensorSonars {
    pub sensor_sonars_mux: u8,
    pub sensor_sonars_err_count: u16,
    pub sensor_sonars_left: f64,
    pub sensor_sonars_middle: f64,
    pub sensor_sonars_right: f64,
    pub sensor_sonars_rear: f64,
    pub sensor_sonars_no_filt_left: f64,
    pub sensor_sonars_no_filt_middle: f64,
    pub sensor_sonars_no_filt_right: f64,
    pub sensor_sonars_no_filt_rear: f64,
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
        let mut msg = Self {
            sensor_sonars_mux,
            sensor_sonars_err_count,
            sensor_sonars_left,
            sensor_sonars_middle,
            sensor_sonars_right,
            sensor_sonars_rear,
            sensor_sonars_no_filt_left,
            sensor_sonars_no_filt_middle,
            sensor_sonars_no_filt_right,
            sensor_sonars_no_filt_rear,
        };
        msg.set_sensor_sonars_mux(msg.sensor_sonars_mux)?;
        msg.set_sensor_sonars_err_count(msg.sensor_sonars_err_count)?;
        msg.set_sensor_sonars_left(msg.sensor_sonars_left)?;
        msg.set_sensor_sonars_middle(msg.sensor_sonars_middle)?;
        msg.set_sensor_sonars_right(msg.sensor_sonars_right)?;
        msg.set_sensor_sonars_rear(msg.sensor_sonars_rear)?;
        msg.set_sensor_sonars_no_filt_left(msg.sensor_sonars_no_filt_left)?;
        msg.set_sensor_sonars_no_filt_middle(msg.sensor_sonars_no_filt_middle)?;
        msg.set_sensor_sonars_no_filt_right(msg.sensor_sonars_no_filt_right)?;
        msg.set_sensor_sonars_no_filt_rear(msg.sensor_sonars_no_filt_rear)?;
        Ok(msg)
    }
    pub fn sensor_sonars_mux(&self) -> u8 {
        self.sensor_sonars_mux
    }
    pub fn sensor_sonars_err_count(&self) -> u16 {
        self.sensor_sonars_err_count
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
    pub fn set_sensor_sonars_mux(&mut self, value: u8) -> Result<(), CanError> {
        if value < 0u8 || value > 0u8 {
            return Err(CanError::ValueOutOfRange);
        }
        self.sensor_sonars_mux = value;
        Ok(())
    }
    pub fn set_sensor_sonars_err_count(&mut self, value: u16) -> Result<(), CanError> {
        if value < 0u16 || value > 0u16 {
            return Err(CanError::ValueOutOfRange);
        }
        self.sensor_sonars_err_count = value;
        Ok(())
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
}
impl CanMessage<{ SensorSonars::LEN }> for SensorSonars {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        if data.len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        let raw_sensor_sonars_mux = data[0usize];
        let raw_sensor_sonars_err_count = u16::from_le_bytes([
            data[0usize],
            data[1usize],
        ]);
        let raw_sensor_sonars_left = u16::from_le_bytes([data[2usize], data[3usize]]);
        let raw_sensor_sonars_middle = u16::from_le_bytes([data[3usize], data[4usize]]);
        let raw_sensor_sonars_right = u16::from_le_bytes([data[5usize], data[6usize]]);
        let raw_sensor_sonars_rear = u16::from_le_bytes([data[6usize], data[7usize]]);
        let raw_sensor_sonars_no_filt_left = u16::from_le_bytes([
            data[2usize],
            data[3usize],
        ]);
        let raw_sensor_sonars_no_filt_middle = u16::from_le_bytes([
            data[3usize],
            data[4usize],
        ]);
        let raw_sensor_sonars_no_filt_right = u16::from_le_bytes([
            data[5usize],
            data[6usize],
        ]);
        let raw_sensor_sonars_no_filt_rear = u16::from_le_bytes([
            data[6usize],
            data[7usize],
        ]);
        Ok(Self {
            sensor_sonars_mux: raw_sensor_sonars_mux as f64 * 1f64,
            sensor_sonars_err_count: raw_sensor_sonars_err_count as f64 * 1f64,
            sensor_sonars_left: raw_sensor_sonars_left as f64 * 0.1f64,
            sensor_sonars_middle: raw_sensor_sonars_middle as f64 * 0.1f64,
            sensor_sonars_right: raw_sensor_sonars_right as f64 * 0.1f64,
            sensor_sonars_rear: raw_sensor_sonars_rear as f64 * 0.1f64,
            sensor_sonars_no_filt_left: raw_sensor_sonars_no_filt_left as f64 * 0.1f64,
            sensor_sonars_no_filt_middle: raw_sensor_sonars_no_filt_middle as f64
                * 0.1f64,
            sensor_sonars_no_filt_right: raw_sensor_sonars_no_filt_right as f64 * 0.1f64,
            sensor_sonars_no_filt_rear: raw_sensor_sonars_no_filt_rear as f64 * 0.1f64,
        })
    }
    fn encode(&self) -> [u8; SensorSonars::LEN] {
        let mut data = [0u8; SensorSonars::LEN];
        data[0usize] = (self.sensor_sonars_mux / 1f64) as u8;
        let bytes = ((self.sensor_sonars_err_count / 1f64) as u16).to_le_bytes();
        data[0usize] = bytes[0usize];
        data[1usize] = bytes[1usize];
        let bytes = ((self.sensor_sonars_left / 0.1f64) as u16).to_le_bytes();
        data[2usize] = bytes[0usize];
        data[3usize] = bytes[1usize];
        let bytes = ((self.sensor_sonars_middle / 0.1f64) as u16).to_le_bytes();
        data[3usize] = bytes[0usize];
        data[4usize] = bytes[1usize];
        let bytes = ((self.sensor_sonars_right / 0.1f64) as u16).to_le_bytes();
        data[5usize] = bytes[0usize];
        data[6usize] = bytes[1usize];
        let bytes = ((self.sensor_sonars_rear / 0.1f64) as u16).to_le_bytes();
        data[6usize] = bytes[0usize];
        data[7usize] = bytes[1usize];
        let bytes = ((self.sensor_sonars_no_filt_left / 0.1f64) as u16).to_le_bytes();
        data[2usize] = bytes[0usize];
        data[3usize] = bytes[1usize];
        let bytes = ((self.sensor_sonars_no_filt_middle / 0.1f64) as u16).to_le_bytes();
        data[3usize] = bytes[0usize];
        data[4usize] = bytes[1usize];
        let bytes = ((self.sensor_sonars_no_filt_right / 0.1f64) as u16).to_le_bytes();
        data[5usize] = bytes[0usize];
        data[6usize] = bytes[1usize];
        let bytes = ((self.sensor_sonars_no_filt_rear / 0.1f64) as u16).to_le_bytes();
        data[6usize] = bytes[0usize];
        data[7usize] = bytes[1usize];
        data
    }
}
