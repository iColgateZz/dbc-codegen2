use crate::ir::{MessageId, ValueDescription, map_into};
use crate::utils::ReprType;
use can_dbc::MessageId as ParsedMessageId;
use can_dbc::ValDescription as ParsedValueDescription;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SignalValueEnumIdx(pub usize);

#[derive(Debug, Clone)]
pub struct SignalValueEnum {
    pub message_id: MessageId,
    pub signal_name: String,
    pub variants: Vec<ValueDescription>,
    pub repr_type: ReprType,
}

impl SignalValueEnum {
    pub fn from_parsed(id: ParsedMessageId, signal_name: String, variants: Vec<ParsedValueDescription>) -> Self {
        Self {
            message_id: id.into(),
            signal_name,
            variants: map_into(variants),
            repr_type: ReprType::default()
        }
    }
}