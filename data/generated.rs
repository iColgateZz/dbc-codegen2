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
pub enum EmptyChoiceEnum {}
impl TryFrom<i8> for EmptyChoiceEnum {
    type Error = CanError;
    fn try_from(val: i8) -> Result<Self, CanError> {
        Ok(
            match val {
                _ => return Err(CanError::IvalidEnumValue),
            },
        )
    }
}
impl From<EmptyChoiceEnum> for i8 {
    fn from(val: EmptyChoiceEnum) -> Self {
        match val {}
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonEmptyChoiceEnum {
    NotAvailable,
    Error,
}
impl TryFrom<i8> for NonEmptyChoiceEnum {
    type Error = CanError;
    fn try_from(val: i8) -> Result<Self, CanError> {
        Ok(
            match val {
                -1i8 => NonEmptyChoiceEnum::NotAvailable,
                -2i8 => NonEmptyChoiceEnum::Error,
                _ => return Err(CanError::IvalidEnumValue),
            },
        )
    }
}
impl From<NonEmptyChoiceEnum> for i8 {
    fn from(val: NonEmptyChoiceEnum) -> Self {
        match val {
            NonEmptyChoiceEnum::NotAvailable => -1i8,
            NonEmptyChoiceEnum::Error => -2i8,
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
///example_message_MSG
///- ID: Standard 10 (0xA)
///- Size: 3 bytes
///- Transmitter: tx_node
#[derive(Debug, Clone)]
pub struct ExampleMessageMsg {
    data: [u8; 3usize],
}
impl ExampleMessageMsg {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(10u16) });
    pub const LEN: usize = 3usize;
    pub fn new(
        no_choice: i8,
        empty_choice: EmptyChoiceEnum,
        non_empty_choice: NonEmptyChoiceEnum,
    ) -> Result<Self, CanError> {
        let mut msg = Self { data: [0u8; Self::LEN] };
        msg.set_no_choice(no_choice)?;
        msg.set_empty_choice(empty_choice)?;
        msg.set_non_empty_choice(non_empty_choice)?;
        Ok(msg)
    }
    ///no_choice
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: VectorXXX
    ///- Start bit: 16
    ///- Size: 8 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: signed
    pub fn no_choice(&self) -> i8 {
        let raw_no_choice = self
            .data
            .view_bits::<Lsb0>()[16usize..24usize]
            .load_le::<i8>();
        (raw_no_choice) * (1i8) + (0i8)
    }
    ///empty_choice
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: VectorXXX
    ///- Start bit: 8
    ///- Size: 8 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: signed
    pub fn empty_choice(&self) -> Result<EmptyChoiceEnum, CanError> {
        let raw_empty_choice = self
            .data
            .view_bits::<Lsb0>()[8usize..16usize]
            .load_le::<i8>();
        Ok(EmptyChoiceEnum::try_from(raw_empty_choice as i8)?)
    }
    ///non_empty_choice
    ///- Min: 0
    ///- Max: 0
    ///- Unit:
    ///- Receivers: VectorXXX
    ///- Start bit: 0
    ///- Size: 8 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: signed
    pub fn non_empty_choice(&self) -> Result<NonEmptyChoiceEnum, CanError> {
        let raw_non_empty_choice = self
            .data
            .view_bits::<Lsb0>()[0usize..8usize]
            .load_le::<i8>();
        Ok(NonEmptyChoiceEnum::try_from(raw_non_empty_choice as i8)?)
    }
    pub fn set_no_choice(&mut self, value: i8) -> Result<(), CanError> {
        if value < 0i8 || value > 0i8 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[16usize..24usize]
            .store_le((value - (0i8)) / (1i8));
        Ok(())
    }
    pub fn set_empty_choice(&mut self, value: EmptyChoiceEnum) -> Result<(), CanError> {
        self.data.view_bits_mut::<Lsb0>()[8usize..16usize].store_le(i8::from(value));
        Ok(())
    }
    pub fn set_non_empty_choice(
        &mut self,
        value: NonEmptyChoiceEnum,
    ) -> Result<(), CanError> {
        self.data.view_bits_mut::<Lsb0>()[0usize..8usize].store_le(i8::from(value));
        Ok(())
    }
}
impl CanMessageTrait<{ Self::LEN }> for ExampleMessageMsg {
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
