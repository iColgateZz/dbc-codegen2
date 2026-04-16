use embedded_can::{Frame, Id, StandardId, ExtendedId};
use bitvec::prelude::*;
use core::ops::BitOr;
#[derive(Debug, Clone)]
pub enum CanError {
    UnknownFrameId,
    UnknownMuxValue,
    InvalidPayloadSize,
    ValueOutOfRange,
    IvalidEnumValue,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnableEnum {
    Disabled,
    D,
}
impl TryFrom<u8> for EnableEnum {
    type Error = CanError;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        Ok(
            match val {
                0u8 => EnableEnum::Disabled,
                1u8 => EnableEnum::D,
                _ => return Err(CanError::IvalidEnumValue),
            },
        )
    }
}
impl From<EnableEnum> for u8 {
    fn from(val: EnableEnum) -> Self {
        match val {
            EnableEnum::Disabled => 0u8,
            EnableEnum::D => 1u8,
        }
    }
}
pub trait CanMessageTrait<const LEN: usize>: Sized {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError>;
    fn encode(&self) -> [u8; LEN];
}
#[derive(Debug, Clone)]
pub enum Msg {
    ExampleMessageMsg(ExampleMessageMsg),
}
impl Msg {
    fn try_from(frame: &impl Frame) -> Result<Self, CanError> {
        let result = match frame.id() {
            ExampleMessageMsg::ID => {
                Msg::ExampleMessageMsg(ExampleMessageMsg::try_from_frame(frame)?)
            }
            _ => return Err(CanError::UnknownFrameId),
        };
        Ok(result)
    }
}
///ExampleMessage_MSG
///- ID: Standard 496 (0x1F0)
///- Size: 8 bytes
///- Transmitter: PCM1
///
///Example message used as template in MotoHawk models.
#[derive(Debug, Clone)]
pub struct ExampleMessageMsg {
    data: [u8; 8usize],
}
impl ExampleMessageMsg {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(496u16) });
    pub const LEN: usize = 8usize;
    pub fn new(
        temperature: f64,
        average_radius: f64,
        enable: EnableEnum,
    ) -> Result<Self, CanError> {
        let mut msg = Self { data: [0u8; Self::LEN] };
        msg.set_temperature(temperature)?;
        msg.set_average_radius(average_radius)?;
        msg.set_enable(enable)?;
        Ok(msg)
    }
    ///Temperature
    ///- Min: 229.52
    ///- Max: 270.47
    ///- Unit: degK
    ///- Receivers: PCM1, FOO
    ///- Start bit: 7
    ///- Size: 12 bits
    ///- Factor: 0.01
    ///- Offset: 250
    ///- Byte order: BigEndian
    ///- Type: signed
    ///
    ///Temperature with a really long and complicated comment that probably require many many lines in a decently wide terminal
    pub fn temperature(&self) -> f64 {
        let raw_temperature = self
            .data
            .view_bits::<Msb0>()[7usize..19usize]
            .load_be::<u16>();
        (raw_temperature as f64) * (0.01f64) + (250f64)
    }
    ///AverageRadius
    ///- Min: 0
    ///- Max: 5
    ///- Unit: m
    ///- Receivers: VectorXXX
    ///- Start bit: 1
    ///- Size: 6 bits
    ///- Factor: 0.1
    ///- Offset: 0
    ///- Byte order: BigEndian
    ///- Type: unsigned
    ///
    ///AverageRadius signal comment
    pub fn average_radius(&self) -> f64 {
        let raw_average_radius = self
            .data
            .view_bits::<Msb0>()[1usize..7usize]
            .load_be::<u8>();
        (raw_average_radius as f64) * (0.1f64) + (0f64)
    }
    ///Enable
    ///- Min: 0
    ///- Max: 0
    ///- Unit: -
    ///- Receivers: VectorXXX
    ///- Start bit: 0
    ///- Size: 1 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: BigEndian
    ///- Type: unsigned
    ///
    ///Enable signal comment
    pub fn enable(&self) -> Result<EnableEnum, CanError> {
        let raw_enable = self.data.view_bits::<Msb0>()[0usize..1usize].load_be::<u8>();
        Ok(EnableEnum::try_from(raw_enable as u8)?)
    }
    pub fn set_temperature(&mut self, value: f64) -> Result<(), CanError> {
        if value < 229.52f64 || value > 270.47f64 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Msb0>()[7usize..19usize]
            .store_be(((value - (250f64)) / (0.01f64)) as u16);
        Ok(())
    }
    pub fn set_average_radius(&mut self, value: f64) -> Result<(), CanError> {
        if value < 0f64 || value > 5f64 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Msb0>()[1usize..7usize]
            .store_be(((value - (0f64)) / (0.1f64)) as u8);
        Ok(())
    }
    pub fn set_enable(&mut self, value: EnableEnum) -> Result<(), CanError> {
        self.data.view_bits_mut::<Msb0>()[0usize..1usize].store_be(u8::from(value));
        Ok(())
    }
}
impl CanMessageTrait<{ Self::LEN }> for ExampleMessageMsg {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {
        let data = frame.data();
        if data.len() < Self::LEN {
            return Err(CanError::InvalidPayloadSize);
        }
        let mut buf = [0u8; 8usize];
        buf.copy_from_slice(&data[..8usize]);
        Ok(Self { data: buf })
    }
    fn encode(&self) -> [u8; Self::LEN] {
        self.data
    }
}
