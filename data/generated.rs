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
pub enum TestInputMux {
    Multiplexer1,
    Multiplexer2,
    Multiplexer3,
    Multiplexer4,
    _Other(u8),
}
impl From<u8> for TestInputMux {
    fn from(val: u8) -> Self {
        match val {
            0u8 => TestInputMux::Multiplexer1,
            1u8 => TestInputMux::Multiplexer2,
            2u8 => TestInputMux::Multiplexer3,
            3u8 => TestInputMux::Multiplexer4,
            _ => TestInputMux::_Other(val),
        }
    }
}
impl From<TestInputMux> for u8 {
    fn from(val: TestInputMux) -> Self {
        match val {
            TestInputMux::Multiplexer1 => 0u8,
            TestInputMux::Multiplexer2 => 1u8,
            TestInputMux::Multiplexer3 => 2u8,
            TestInputMux::Multiplexer4 => 3u8,
            TestInputMux::_Other(v) => v,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestOutputMux {
    Multiplexer1Out,
    Multiplexer2Out,
    Multiplexer3Out,
    Multiplexer4Out,
    _Other(u8),
}
impl From<u8> for TestOutputMux {
    fn from(val: u8) -> Self {
        match val {
            0u8 => TestOutputMux::Multiplexer1Out,
            1u8 => TestOutputMux::Multiplexer2Out,
            2u8 => TestOutputMux::Multiplexer3Out,
            3u8 => TestOutputMux::Multiplexer4Out,
            _ => TestOutputMux::_Other(val),
        }
    }
}
impl From<TestOutputMux> for u8 {
    fn from(val: TestOutputMux) -> Self {
        match val {
            TestOutputMux::Multiplexer1Out => 0u8,
            TestOutputMux::Multiplexer2Out => 1u8,
            TestOutputMux::Multiplexer3Out => 2u8,
            TestOutputMux::Multiplexer4Out => 3u8,
            TestOutputMux::_Other(v) => v,
        }
    }
}
pub trait CanMessageTrait<const LEN: usize>: Sized {
    fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError>;
    fn encode(&self) -> [u8; LEN];
}
#[derive(Debug, Clone)]
pub enum Msg {
    TestInputMsg(TestInputMsg),
    TestOutputMsg(TestOutputMsg),
}
impl Msg {
    fn try_from(frame: &impl Frame) -> Result<Self, CanError> {
        let result = match frame.id() {
            TestInputMsg::ID => Msg::TestInputMsg(TestInputMsg::try_from_frame(frame)?),
            TestOutputMsg::ID => {
                Msg::TestOutputMsg(TestOutputMsg::try_from_frame(frame)?)
            }
            _ => return Err(CanError::UnknownFrameId),
        };
        Ok(result)
    }
}
#[derive(Debug, Clone)]
pub enum TestInputMsgMux {
    V0(TestInputMsgMux0),
    V1(TestInputMsgMux1),
    V2(TestInputMsgMux2),
    V3(TestInputMsgMux3),
}
#[derive(Debug, Clone, Default)]
pub struct TestInputMsgMux0 {
    data: [u8; 8usize],
}
impl TestInputMsgMux0 {
    pub fn new() -> Self {
        Self { data: [0u8; 8usize] }
    }
    ///Var1
    ///- Min: 0
    ///- Max: 65535
    ///- Unit:
    ///- Receivers: VectorXXX
    ///- Start bit: 0
    ///- Size: 16 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn var1(&self) -> u16 {
        let raw_var1 = self.data.view_bits::<Lsb0>()[0usize..16usize].load_le::<u16>();
        (raw_var1) * (1u16) + (0u16)
    }
    pub fn set_var1(&mut self, value: u16) -> Result<(), CanError> {
        if value < 0u16 || value > 65535u16 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[0usize..16usize]
            .store_le((value - (0u16)) / (1u16));
        Ok(())
    }
}
#[derive(Debug, Clone, Default)]
pub struct TestInputMsgMux1 {
    data: [u8; 8usize],
}
impl TestInputMsgMux1 {
    pub fn new() -> Self {
        Self { data: [0u8; 8usize] }
    }
    ///Var2
    ///- Min: 0
    ///- Max: 65535
    ///- Unit:
    ///- Receivers: VectorXXX
    ///- Start bit: 0
    ///- Size: 16 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn var2(&self) -> u16 {
        let raw_var2 = self.data.view_bits::<Lsb0>()[0usize..16usize].load_le::<u16>();
        (raw_var2) * (1u16) + (0u16)
    }
    pub fn set_var2(&mut self, value: u16) -> Result<(), CanError> {
        if value < 0u16 || value > 65535u16 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[0usize..16usize]
            .store_le((value - (0u16)) / (1u16));
        Ok(())
    }
}
#[derive(Debug, Clone, Default)]
pub struct TestInputMsgMux2 {
    data: [u8; 8usize],
}
impl TestInputMsgMux2 {
    pub fn new() -> Self {
        Self { data: [0u8; 8usize] }
    }
    ///Var3
    ///- Min: 0
    ///- Max: 65535
    ///- Unit:
    ///- Receivers: VectorXXX
    ///- Start bit: 0
    ///- Size: 16 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn var3(&self) -> u16 {
        let raw_var3 = self.data.view_bits::<Lsb0>()[0usize..16usize].load_le::<u16>();
        (raw_var3) * (1u16) + (0u16)
    }
    pub fn set_var3(&mut self, value: u16) -> Result<(), CanError> {
        if value < 0u16 || value > 65535u16 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[0usize..16usize]
            .store_le((value - (0u16)) / (1u16));
        Ok(())
    }
}
#[derive(Debug, Clone, Default)]
pub struct TestInputMsgMux3 {
    data: [u8; 8usize],
}
impl TestInputMsgMux3 {
    pub fn new() -> Self {
        Self { data: [0u8; 8usize] }
    }
    ///Var4
    ///- Min: 0
    ///- Max: 65535
    ///- Unit:
    ///- Receivers: VectorXXX
    ///- Start bit: 0
    ///- Size: 16 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn var4(&self) -> u16 {
        let raw_var4 = self.data.view_bits::<Lsb0>()[0usize..16usize].load_le::<u16>();
        (raw_var4) * (1u16) + (0u16)
    }
    pub fn set_var4(&mut self, value: u16) -> Result<(), CanError> {
        if value < 0u16 || value > 65535u16 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[0usize..16usize]
            .store_le((value - (0u16)) / (1u16));
        Ok(())
    }
}
///Test_input_MSG
///- ID: Standard 0 (0x0)
///- Size: 8 bytes
///- Transmitter: VectorXXX
#[derive(Debug, Clone)]
pub struct TestInputMsg {
    data: [u8; 8usize],
}
impl TestInputMsg {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(0u16) });
    pub const LEN: usize = 8usize;
    pub fn mux(&self) -> Result<TestInputMsgMux, CanError> {
        let raw_test_input_mux = self
            .data
            .view_bits::<Lsb0>()[56usize..60usize]
            .load_le::<u8>();
        match raw_test_input_mux {
            0 => {
                Ok(
                    TestInputMsgMux::V0(TestInputMsgMux0 {
                        data: self.data,
                    }),
                )
            }
            1 => {
                Ok(
                    TestInputMsgMux::V1(TestInputMsgMux1 {
                        data: self.data,
                    }),
                )
            }
            2 => {
                Ok(
                    TestInputMsgMux::V2(TestInputMsgMux2 {
                        data: self.data,
                    }),
                )
            }
            3 => {
                Ok(
                    TestInputMsgMux::V3(TestInputMsgMux3 {
                        data: self.data,
                    }),
                )
            }
            _ => Err(CanError::UnknownMuxValue),
        }
    }
    pub fn set_mux_0(&mut self, value: TestInputMsgMux0) -> Result<(), CanError> {
        let b0 = BitArray::<_, LocalBits>::new(self.data);
        let b1 = BitArray::<_, LocalBits>::new(value.data);
        self.data = b0.bitor(b1).into_inner();
        self.data.view_bits_mut::<Lsb0>()[56usize..60usize].store_le(0u64 as u8);
        Ok(())
    }
    pub fn set_mux_1(&mut self, value: TestInputMsgMux1) -> Result<(), CanError> {
        let b0 = BitArray::<_, LocalBits>::new(self.data);
        let b1 = BitArray::<_, LocalBits>::new(value.data);
        self.data = b0.bitor(b1).into_inner();
        self.data.view_bits_mut::<Lsb0>()[56usize..60usize].store_le(1u64 as u8);
        Ok(())
    }
    pub fn set_mux_2(&mut self, value: TestInputMsgMux2) -> Result<(), CanError> {
        let b0 = BitArray::<_, LocalBits>::new(self.data);
        let b1 = BitArray::<_, LocalBits>::new(value.data);
        self.data = b0.bitor(b1).into_inner();
        self.data.view_bits_mut::<Lsb0>()[56usize..60usize].store_le(2u64 as u8);
        Ok(())
    }
    pub fn set_mux_3(&mut self, value: TestInputMsgMux3) -> Result<(), CanError> {
        let b0 = BitArray::<_, LocalBits>::new(self.data);
        let b1 = BitArray::<_, LocalBits>::new(value.data);
        self.data = b0.bitor(b1).into_inner();
        self.data.view_bits_mut::<Lsb0>()[56usize..60usize].store_le(3u64 as u8);
        Ok(())
    }
}
impl CanMessageTrait<{ Self::LEN }> for TestInputMsg {
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
#[derive(Debug, Clone)]
pub enum TestOutputMsgMux {
    V0(TestOutputMsgMux0),
    V1(TestOutputMsgMux1),
    V2(TestOutputMsgMux2),
    V3(TestOutputMsgMux3),
}
#[derive(Debug, Clone, Default)]
pub struct TestOutputMsgMux0 {
    data: [u8; 8usize],
}
impl TestOutputMsgMux0 {
    pub fn new() -> Self {
        Self { data: [0u8; 8usize] }
    }
    ///Var5
    ///- Min: 0
    ///- Max: 65535
    ///- Unit:
    ///- Receivers: VectorXXX
    ///- Start bit: 0
    ///- Size: 16 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn var5(&self) -> u16 {
        let raw_var5 = self.data.view_bits::<Lsb0>()[0usize..16usize].load_le::<u16>();
        (raw_var5) * (1u16) + (0u16)
    }
    pub fn set_var5(&mut self, value: u16) -> Result<(), CanError> {
        if value < 0u16 || value > 65535u16 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[0usize..16usize]
            .store_le((value - (0u16)) / (1u16));
        Ok(())
    }
}
#[derive(Debug, Clone, Default)]
pub struct TestOutputMsgMux1 {
    data: [u8; 8usize],
}
impl TestOutputMsgMux1 {
    pub fn new() -> Self {
        Self { data: [0u8; 8usize] }
    }
    ///Var6
    ///- Min: 0
    ///- Max: 65535
    ///- Unit:
    ///- Receivers: VectorXXX
    ///- Start bit: 0
    ///- Size: 16 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn var6(&self) -> u16 {
        let raw_var6 = self.data.view_bits::<Lsb0>()[0usize..16usize].load_le::<u16>();
        (raw_var6) * (1u16) + (0u16)
    }
    pub fn set_var6(&mut self, value: u16) -> Result<(), CanError> {
        if value < 0u16 || value > 65535u16 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[0usize..16usize]
            .store_le((value - (0u16)) / (1u16));
        Ok(())
    }
}
#[derive(Debug, Clone, Default)]
pub struct TestOutputMsgMux2 {
    data: [u8; 8usize],
}
impl TestOutputMsgMux2 {
    pub fn new() -> Self {
        Self { data: [0u8; 8usize] }
    }
    ///Var7
    ///- Min: 0
    ///- Max: 65535
    ///- Unit:
    ///- Receivers: VectorXXX
    ///- Start bit: 0
    ///- Size: 16 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn var7(&self) -> u16 {
        let raw_var7 = self.data.view_bits::<Lsb0>()[0usize..16usize].load_le::<u16>();
        (raw_var7) * (1u16) + (0u16)
    }
    pub fn set_var7(&mut self, value: u16) -> Result<(), CanError> {
        if value < 0u16 || value > 65535u16 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[0usize..16usize]
            .store_le((value - (0u16)) / (1u16));
        Ok(())
    }
}
#[derive(Debug, Clone, Default)]
pub struct TestOutputMsgMux3 {
    data: [u8; 8usize],
}
impl TestOutputMsgMux3 {
    pub fn new() -> Self {
        Self { data: [0u8; 8usize] }
    }
    ///Var8
    ///- Min: 0
    ///- Max: 65535
    ///- Unit:
    ///- Receivers: VectorXXX
    ///- Start bit: 0
    ///- Size: 16 bits
    ///- Factor: 1
    ///- Offset: 0
    ///- Byte order: LittleEndian
    ///- Type: unsigned
    pub fn var8(&self) -> u16 {
        let raw_var8 = self.data.view_bits::<Lsb0>()[0usize..16usize].load_le::<u16>();
        (raw_var8) * (1u16) + (0u16)
    }
    pub fn set_var8(&mut self, value: u16) -> Result<(), CanError> {
        if value < 0u16 || value > 65535u16 {
            return Err(CanError::ValueOutOfRange);
        }
        self.data
            .view_bits_mut::<Lsb0>()[0usize..16usize]
            .store_le((value - (0u16)) / (1u16));
        Ok(())
    }
}
///Test_output_MSG
///- ID: Standard 1 (0x1)
///- Size: 8 bytes
///- Transmitter: VectorXXX
#[derive(Debug, Clone)]
pub struct TestOutputMsg {
    data: [u8; 8usize],
}
impl TestOutputMsg {
    pub const ID: Id = Id::Standard(unsafe { StandardId::new_unchecked(1u16) });
    pub const LEN: usize = 8usize;
    pub fn mux(&self) -> Result<TestOutputMsgMux, CanError> {
        let raw_test_output_mux = self
            .data
            .view_bits::<Lsb0>()[56usize..60usize]
            .load_le::<u8>();
        match raw_test_output_mux {
            0 => {
                Ok(
                    TestOutputMsgMux::V0(TestOutputMsgMux0 {
                        data: self.data,
                    }),
                )
            }
            1 => {
                Ok(
                    TestOutputMsgMux::V1(TestOutputMsgMux1 {
                        data: self.data,
                    }),
                )
            }
            2 => {
                Ok(
                    TestOutputMsgMux::V2(TestOutputMsgMux2 {
                        data: self.data,
                    }),
                )
            }
            3 => {
                Ok(
                    TestOutputMsgMux::V3(TestOutputMsgMux3 {
                        data: self.data,
                    }),
                )
            }
            _ => Err(CanError::UnknownMuxValue),
        }
    }
    pub fn set_mux_0(&mut self, value: TestOutputMsgMux0) -> Result<(), CanError> {
        let b0 = BitArray::<_, LocalBits>::new(self.data);
        let b1 = BitArray::<_, LocalBits>::new(value.data);
        self.data = b0.bitor(b1).into_inner();
        self.data.view_bits_mut::<Lsb0>()[56usize..60usize].store_le(0u64 as u8);
        Ok(())
    }
    pub fn set_mux_1(&mut self, value: TestOutputMsgMux1) -> Result<(), CanError> {
        let b0 = BitArray::<_, LocalBits>::new(self.data);
        let b1 = BitArray::<_, LocalBits>::new(value.data);
        self.data = b0.bitor(b1).into_inner();
        self.data.view_bits_mut::<Lsb0>()[56usize..60usize].store_le(1u64 as u8);
        Ok(())
    }
    pub fn set_mux_2(&mut self, value: TestOutputMsgMux2) -> Result<(), CanError> {
        let b0 = BitArray::<_, LocalBits>::new(self.data);
        let b1 = BitArray::<_, LocalBits>::new(value.data);
        self.data = b0.bitor(b1).into_inner();
        self.data.view_bits_mut::<Lsb0>()[56usize..60usize].store_le(2u64 as u8);
        Ok(())
    }
    pub fn set_mux_3(&mut self, value: TestOutputMsgMux3) -> Result<(), CanError> {
        let b0 = BitArray::<_, LocalBits>::new(self.data);
        let b1 = BitArray::<_, LocalBits>::new(value.data);
        self.data = b0.bitor(b1).into_inner();
        self.data.view_bits_mut::<Lsb0>()[56usize..60usize].store_le(3u64 as u8);
        Ok(())
    }
}
impl CanMessageTrait<{ Self::LEN }> for TestOutputMsg {
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
