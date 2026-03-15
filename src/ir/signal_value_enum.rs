use crate::ir::{MessageId, ValueDescriptionIdx};
use crate::utils::ReprType;
use can_dbc::MessageId as ParsedMessageId;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SignalValueEnumIdx(pub usize);

#[derive(Debug, Clone)]
pub struct SignalValueEnum {
    pub message_id: MessageId,
    pub signal_name: String,
    pub variants: Vec<ValueDescriptionIdx>,
    pub repr_type: ReprType,
}

impl SignalValueEnum {
    pub fn from_parsed(id: ParsedMessageId, signal_name: String, variants: Vec<ValueDescriptionIdx>) -> Self {
        Self {
            message_id: id.into(),
            signal_name,
            variants,
            repr_type: ReprType::default()
        }
    }
}