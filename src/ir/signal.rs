use crate::ir::{NodeName, SignalValueEnum, map_into};
use can_dbc::ByteOrder as ParsedByteOrder;
use can_dbc::MultiplexIndicator as ParsedMultiplexIndicator;
use can_dbc::Signal as ParsedSignal;
use can_dbc::ValueType as ParsedValueType;
use crate::utils::ToUpperCamelCase;

#[derive(Debug, Clone)]
pub struct Signal {
    pub name: SignalName,
    pub multiplexer: MultiplexIndicator,
    pub start_bit: u64,
    pub size: u64,
    pub byte_order: ByteOrder,
    pub value_type: ValueType,
    pub factor: f64,
    pub offset: f64,
    pub min: f64,
    pub max: f64,
    pub unit: String,
    pub receivers: Vec<Receiver>,
    pub signal_value_enum: Option<SignalValueEnum>,
}
impl From<ParsedSignal> for Signal {
    fn from(value: ParsedSignal) -> Self {
        Signal {
            name: SignalName(value.name),
            multiplexer: MultiplexIndicator::from(value.multiplexer_indicator),
            start_bit: value.start_bit,
            size: value.size,
            byte_order: ByteOrder::from(value.byte_order),
            value_type: ValueType::from(value.value_type),
            factor: value.factor,
            offset: value.offset,
            min: value.min,
            max: value.max,
            unit: value.unit,
            receivers: map_into(value.receivers),
            signal_value_enum: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SignalName(String);

impl SignalName {
    pub fn raw(&self) -> &str {
        &self.0
    }

    pub fn lower(&self) -> String {
        self.0.to_lowercase()
    }

    pub fn upper_camel(&self) -> String {
        self.0.to_upper_camelcase()
    }
}

#[derive(Debug, Clone)]
pub enum MultiplexIndicator {
    Multiplexor,
    MultiplexedSignal(u64),
    MultiplexorAndMultiplexedSignal(u64),
    Plain,
}
impl From<ParsedMultiplexIndicator> for MultiplexIndicator {
    fn from(value: ParsedMultiplexIndicator) -> Self {
        match value {
            ParsedMultiplexIndicator::Multiplexor => MultiplexIndicator::Multiplexor,
            ParsedMultiplexIndicator::MultiplexedSignal(v) => {
                MultiplexIndicator::MultiplexedSignal(v)
            }
            ParsedMultiplexIndicator::MultiplexorAndMultiplexedSignal(v) => {
                MultiplexIndicator::MultiplexorAndMultiplexedSignal(v)
            }
            ParsedMultiplexIndicator::Plain => MultiplexIndicator::Plain,
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Receiver {
    Node(NodeName),
    VectorXXX,
}
impl From<String> for Receiver {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Vector__XXX" => Receiver::VectorXXX,
            _ => Receiver::Node(NodeName(value)),
        }
    }
}
