use crate::ir::{MessageId, map_into};
use crate::utils::ReprType;
use can_dbc::ValDescription as ParsedValDescription;
use can_dbc::ValueDescription as ParsedValueDescription;

#[derive(Debug, Clone)]
pub struct SignalValueEnum {
    pub message_id: MessageId,
    pub signal_name: String,
    pub variants: Vec<ValueDescription>,
    pub repr_type: ReprType,
}

#[derive(Debug, Clone)]
pub struct ValueDescription {
    pub value: i64,
    pub description: String,
}

impl TryFrom<ParsedValueDescription> for SignalValueEnum {
    type Error = ();

    fn try_from(v: ParsedValueDescription) -> Result<Self, Self::Error> {
        match v {
            ParsedValueDescription::Signal {
                message_id,
                name,
                value_descriptions,
            } => Ok(Self {
                message_id: message_id.into(),
                signal_name: name,
                variants: map_into(value_descriptions),
                repr_type: ReprType::default(),
            }),
            _ => Err(()),
        }
    }
}

impl From<ParsedValDescription> for ValueDescription {
    fn from(value: ParsedValDescription) -> Self {
        Self {
            value: value.id,
            description: value.description,
        }
    }
}
