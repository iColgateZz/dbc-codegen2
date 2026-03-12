use crate::ir::{MessageId, map_into};
use can_dbc::ValueDescription as ParsedValueDescription;
use can_dbc::ValDescription as ParsedValDescription;

#[derive(Debug, Clone)]
pub struct SignalValueEnum {
    message_id: MessageId,
    signal_name: String,
    variants: Vec<ValueDescription>,
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