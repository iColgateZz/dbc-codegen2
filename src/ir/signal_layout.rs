use can_dbc::ByteOrder as ParsedByteOrder;
use can_dbc::ValueType as ParsedValueType;
use can_dbc::Signal as ParsedSignal;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SignalLayoutIdx(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct SignalLayout {
    pub start_bit: u64,
    pub size: u64,
    pub byte_order: ByteOrder,
    pub value_type: ValueType,
    pub factor: f64,
    pub offset: f64,
    pub min: f64,
    pub max: f64,

    /// Pre-computed by `ComputeBitvecPositions` transformation node.
    pub bitvec_start: usize,
    pub bitvec_end: usize,
}

impl From<&ParsedSignal> for SignalLayout {
    fn from(value: &ParsedSignal) -> Self {
        Self {
            start_bit: value.start_bit,
            size: value.size,
            byte_order: ByteOrder::from(value.byte_order),
            value_type: ValueType::from(value.value_type),
            factor: value.factor,
            offset: value.offset,
            min: value.min,
            max: value.max,
            bitvec_start: 0,
            bitvec_end: 0,
        }
    }
}

use std::hash::{Hash, Hasher};

impl PartialEq for SignalLayout {
    fn eq(&self, other: &Self) -> bool {
        self.start_bit == other.start_bit
            && self.size == other.size
            && self.byte_order == other.byte_order
            && self.value_type == other.value_type
            && self.factor.to_bits() == other.factor.to_bits()
            && self.offset.to_bits() == other.offset.to_bits()
            && self.min.to_bits() == other.min.to_bits()
            && self.max.to_bits() == other.max.to_bits()
    }
}

impl Eq for SignalLayout {}

impl Hash for SignalLayout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start_bit.hash(state);
        self.size.hash(state);
        self.byte_order.hash(state);
        self.value_type.hash(state);

        self.factor.to_bits().hash(state);
        self.offset.to_bits().hash(state);
        self.min.to_bits().hash(state);
        self.max.to_bits().hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}
impl From<ParsedByteOrder> for ByteOrder {
    fn from(value: ParsedByteOrder) -> Self {
        match value {
            ParsedByteOrder::BigEndian => ByteOrder::BigEndian,
            ParsedByteOrder::LittleEndian => ByteOrder::LittleEndian,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    Unsigned,
    Signed,
}
impl From<ParsedValueType> for ValueType {
    fn from(value: ParsedValueType) -> Self {
        match value {
            ParsedValueType::Signed => ValueType::Signed,
            ParsedValueType::Unsigned => ValueType::Unsigned,
        }
    }
}