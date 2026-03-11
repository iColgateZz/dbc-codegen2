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
            _ => return Err(CanError::Err1),
        };
        Ok(result)
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
