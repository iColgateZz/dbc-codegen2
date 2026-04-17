use crate::ir::identifier::Identifier;
use crate::ir::map_into;
use crate::ir::signal_layout::SignalLayoutIdx;
use crate::ir::signal_value_type::IntReprType;
use can_dbc::MultiplexIndicator as ParsedMultiplexIndicator;
use can_dbc::Signal as ParsedSignal;
use crate::ir::signal_value_type::{PhysicalType, RawType};
use crate::ir::{SignalValueEnumIdx, ExtendedValueType};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SignalIdx(pub usize);

#[derive(Debug, Clone)]
pub struct Signal {
    pub name: Identifier,
    pub multiplexer: MultiplexIndicator,
    pub unit: String,
    pub receivers: Vec<Receiver>,

    pub layout: SignalLayoutIdx,
    pub signal_value_enum_idx: Option<SignalValueEnumIdx>,
    pub extended_type: ExtendedValueType,

    pub raw_type: RawType,
    pub physical_type: PhysicalType,

    pub comment: Option<String>,
}

impl From<ParsedSignal> for Signal {
    fn from(value: ParsedSignal) -> Self {
        Signal {
            name: Identifier::from_raw(value.name),
            multiplexer: MultiplexIndicator::from(value.multiplexer_indicator),
            unit: value.unit,
            receivers: map_into(value.receivers),

            layout: SignalLayoutIdx(0),
            signal_value_enum_idx: None,
            extended_type: ExtendedValueType::Integer,

            raw_type: RawType::Integer(IntReprType::I64),
            physical_type: PhysicalType::Integer(IntReprType::I64),

            comment: None,
        }
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
pub enum Receiver {
    Node(Identifier),
    VectorXXX,
}
impl From<String> for Receiver {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Vector__XXX" => Receiver::VectorXXX,
            _ => Receiver::Node(Identifier::from_raw(value)),
        }
    }
}
