use embedded_can::{Frame, Id, StandardId, ExtendedId};
#[derive(Debug, Clone)]
pub enum CanError {
    Err1,
    Err2,
}
pub trait CanMessage<const LEN: usize>: Sized {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError>;
    fn encode(&self) -> (Id, [u8; LEN]);
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
        let id = match frame.id() {
            Id::Standard(sid) => sid.as_raw() as u32,
            Id::Extended(eid) => eid.as_raw(),
        };
        let result = match id {
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
            2i64 => Self::Reboot,
            1i64 => Self::Sync,
            0i64 => Self::Noop,
            _ => Self::_Other(val),
        }
    }
}
impl From<DriverHeartbeatCmd> for u8 {
    fn from(val: DriverHeartbeatCmd) -> Self {
        match val {
            DriverHeartbeatCmd::Reboot => 2i64,
            DriverHeartbeatCmd::Sync => 1i64,
            DriverHeartbeatCmd::Noop => 0i64,
            DriverHeartbeatCmd::_Other(v) => v,
        }
    }
}
#[derive(Debug, Clone)]
pub struct DriverHeartbeat {
    pub driver_heartbeat_cmd: f64,
}
impl DriverHeartbeat {
    pub const ID: u32 = 100u32;
    pub const LEN: usize = 1usize;
}
impl CanMessage<{ DriverHeartbeat::LEN }> for DriverHeartbeat {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        let raw_driver_heartbeat_cmd = u16::from_le_bytes([data[0usize], data[1usize]]);
        Ok(Self {
            driver_heartbeat_cmd: raw_driver_heartbeat_cmd as f64 * 1f64,
        })
    }
    fn encode(&self) -> (Id, [u8; DriverHeartbeat::LEN]) {
        let mut data = [0u8; DriverHeartbeat::LEN];
        let id = Id::Standard(StandardId::new(Self::ID as u16).unwrap());
        (id, data)
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
            2i64 => Self::IoDebugTest2EnumTwo,
            1i64 => Self::IoDebugTest2EnumOne,
            _ => Self::_Other(val),
        }
    }
}
impl From<IoDebugTestEnum> for u8 {
    fn from(val: IoDebugTestEnum) -> Self {
        match val {
            IoDebugTestEnum::IoDebugTest2EnumTwo => 2i64,
            IoDebugTestEnum::IoDebugTest2EnumOne => 1i64,
            IoDebugTestEnum::_Other(v) => v,
        }
    }
}
#[derive(Debug, Clone)]
pub struct IoDebug {
    pub io_debug_test_unsigned: f64,
    pub io_debug_test_enum: f64,
    pub io_debug_test_signed: f64,
    pub io_debug_test_float: f64,
}
impl IoDebug {
    pub const ID: u32 = 500u32;
    pub const LEN: usize = 4usize;
}
impl CanMessage<{ IoDebug::LEN }> for IoDebug {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        let raw_io_debug_test_unsigned = u16::from_le_bytes([
            data[0usize],
            data[1usize],
        ]);
        let raw_io_debug_test_enum = u16::from_le_bytes([data[2usize], data[3usize]]);
        let raw_io_debug_test_signed = u16::from_le_bytes([data[4usize], data[5usize]]);
        let raw_io_debug_test_float = u16::from_le_bytes([data[6usize], data[7usize]]);
        Ok(Self {
            io_debug_test_unsigned: raw_io_debug_test_unsigned as f64 * 1f64,
            io_debug_test_enum: raw_io_debug_test_enum as f64 * 1f64,
            io_debug_test_signed: raw_io_debug_test_signed as f64 * 1f64,
            io_debug_test_float: raw_io_debug_test_float as f64 * 0.5f64,
        })
    }
    fn encode(&self) -> (Id, [u8; IoDebug::LEN]) {
        let mut data = [0u8; IoDebug::LEN];
        let id = Id::Standard(StandardId::new(Self::ID as u16).unwrap());
        (id, data)
    }
}
#[derive(Debug, Clone)]
pub struct MotorCmd {
    pub motor_cmd_steer: f64,
    pub motor_cmd_drive: f64,
}
impl MotorCmd {
    pub const ID: u32 = 101u32;
    pub const LEN: usize = 1usize;
}
impl CanMessage<{ MotorCmd::LEN }> for MotorCmd {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        let raw_motor_cmd_steer = u16::from_le_bytes([data[0usize], data[1usize]]);
        let raw_motor_cmd_drive = u16::from_le_bytes([data[2usize], data[3usize]]);
        Ok(Self {
            motor_cmd_steer: raw_motor_cmd_steer as f64 * 1f64,
            motor_cmd_drive: raw_motor_cmd_drive as f64 * 1f64,
        })
    }
    fn encode(&self) -> (Id, [u8; MotorCmd::LEN]) {
        let mut data = [0u8; MotorCmd::LEN];
        let id = Id::Standard(StandardId::new(Self::ID as u16).unwrap());
        (id, data)
    }
}
#[derive(Debug, Clone)]
pub struct MotorStatus {
    pub motor_status_wheel_error: f64,
    pub motor_status_speed_kph: f64,
}
impl MotorStatus {
    pub const ID: u32 = 400u32;
    pub const LEN: usize = 3usize;
}
impl CanMessage<{ MotorStatus::LEN }> for MotorStatus {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        let raw_motor_status_wheel_error = u16::from_le_bytes([
            data[0usize],
            data[1usize],
        ]);
        let raw_motor_status_speed_kph = u16::from_le_bytes([
            data[2usize],
            data[3usize],
        ]);
        Ok(Self {
            motor_status_wheel_error: raw_motor_status_wheel_error as f64 * 1f64,
            motor_status_speed_kph: raw_motor_status_speed_kph as f64 * 0.001f64,
        })
    }
    fn encode(&self) -> (Id, [u8; MotorStatus::LEN]) {
        let mut data = [0u8; MotorStatus::LEN];
        let id = Id::Standard(StandardId::new(Self::ID as u16).unwrap());
        (id, data)
    }
}
#[derive(Debug, Clone)]
pub struct SensorSonars {
    pub sensor_sonars_mux: f64,
    pub sensor_sonars_err_count: f64,
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
    pub const ID: u32 = 200u32;
    pub const LEN: usize = 8usize;
}
impl CanMessage<{ SensorSonars::LEN }> for SensorSonars {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        let raw_sensor_sonars_mux = u16::from_le_bytes([data[0usize], data[1usize]]);
        let raw_sensor_sonars_err_count = u16::from_le_bytes([
            data[2usize],
            data[3usize],
        ]);
        let raw_sensor_sonars_left = u16::from_le_bytes([data[4usize], data[5usize]]);
        let raw_sensor_sonars_middle = u16::from_le_bytes([data[6usize], data[7usize]]);
        let raw_sensor_sonars_right = u16::from_le_bytes([data[8usize], data[9usize]]);
        let raw_sensor_sonars_rear = u16::from_le_bytes([data[10usize], data[11usize]]);
        let raw_sensor_sonars_no_filt_left = u16::from_le_bytes([
            data[12usize],
            data[13usize],
        ]);
        let raw_sensor_sonars_no_filt_middle = u16::from_le_bytes([
            data[14usize],
            data[15usize],
        ]);
        let raw_sensor_sonars_no_filt_right = u16::from_le_bytes([
            data[16usize],
            data[17usize],
        ]);
        let raw_sensor_sonars_no_filt_rear = u16::from_le_bytes([
            data[18usize],
            data[19usize],
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
    fn encode(&self) -> (Id, [u8; SensorSonars::LEN]) {
        let mut data = [0u8; SensorSonars::LEN];
        let id = Id::Standard(StandardId::new(Self::ID as u16).unwrap());
        (id, data)
    }
}
