use embedded_can::{ExtendedId, Frame, Id, StandardId};

#[derive(Debug, Clone)]
pub enum CanError {
    Err1,
    Err2,
    // ...
}

pub trait CanMessage<const LEN: usize>: Sized {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError>;
    fn encode(&self) -> (Id, [u8; LEN]);
}

#[derive(Debug, Clone)]
pub enum Msg {
    EngineData(EngineData),
    OtherData(OtherData),
}

impl Msg {
    fn try_from(frame: &impl Frame) -> Result<Self, CanError> {
        let id = match frame.id() {
            Id::Standard(sid) => sid.as_raw() as u32,
            Id::Extended(eid) => eid.as_raw(),
        };

        let result = match id {
            EngineData::ID => Msg::EngineData(EngineData::try_from_frame(frame)?),
            OtherData::ID => Msg::OtherData(OtherData::try_from_frame(frame)?),
            _ => return Err(CanError::Err1),
        };

        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum EngineMode {
    Off   = 0,
    Idle  = 1,
    Drive = 2,
    Sport = 3,
    _Other(u8),
}

impl From<u8> for EngineMode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Off,
            1 => Self::Idle,
            2 => Self::Drive,
            3 => Self::Sport,
            _ => Self::_Other(value),
        }
    }
}

impl From<EngineMode> for u8 {
    fn from(val: EngineMode) -> u8 {
        match val {
            EngineMode::Off        => 0,
            EngineMode::Idle       => 1,
            EngineMode::Drive      => 2,
            EngineMode::Sport      => 3,
            EngineMode::_Other(v)  => v,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineData {
    pub rpm: f32,
    pub speed: f32,
    pub engine_mode: EngineMode,
}

impl EngineData {
    const ID: u32 = 100;
    const LEN: usize = 8;
}

impl CanMessage<{ EngineData::LEN }> for EngineData {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();

        let raw_rpm = u16::from_le_bytes([data[0], data[1]]);
        let raw_speed = u16::from_le_bytes([data[2], data[3]]);

        Ok(Self {
            rpm: raw_rpm as f32 * 0.125,
            speed: raw_speed as f32 * 0.01,
            engine_mode: EngineMode::from(data[4]),
        })
    }

    fn encode(&self) -> (Id, [u8; EngineData::LEN]) {
        let mut data = [0u8; EngineData::LEN];

        let raw_rpm = (self.rpm / 0.125) as u16;
        let raw_speed = (self.speed / 0.01) as u16;

        let rpm_bytes = raw_rpm.to_le_bytes();
        let speed_bytes = raw_speed.to_le_bytes();

        data[0] = rpm_bytes[0];
        data[1] = rpm_bytes[1];
        data[2] = speed_bytes[0];
        data[3] = speed_bytes[1];
        data[4] = u8::from(self.engine_mode.clone());

        let id = Id::Extended(ExtendedId::new(Self::ID).unwrap());
        (id, data)
    }
}

#[derive(Debug, Clone)]
pub struct OtherData {
    pub something: f32,
}

impl OtherData {
    const ID: u32 = 101;
    const LEN: usize = 8;
}

impl CanMessage<{ OtherData::LEN }> for OtherData {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        // some logic
        Ok(
            Self {
                something: 5.0
            }
        )
    }

    // other things ...
    fn encode(&self) -> (Id, [u8; Self::LEN]) {
        let data = [0u8; Self::LEN];

        //some data manipulations

        let id = Id::Standard(StandardId::new(Self::ID as u16).unwrap());

        (id, data)
    }
}
