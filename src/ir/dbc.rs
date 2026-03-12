use crate::ir::{Message, Node, map_into, SignalValueEnum};
use can_dbc::Dbc as ParsedDbc;

#[derive(Debug, Clone)]
pub struct DbcFile {
    pub nodes: Vec<Node>,
    pub messages: Vec<Message>,
    pub signal_value_enums: Vec<SignalValueEnum>,
}

impl DbcFile {
    pub fn from_dbc(dbc: ParsedDbc) -> Self {
        DbcFile::from(dbc)
    }
}

impl From<ParsedDbc> for DbcFile {
    fn from(value: ParsedDbc) -> Self {
        let enums = value.value_descriptions
            .into_iter()
            .filter_map(|v| SignalValueEnum::try_from(v).ok())
            .collect();

        DbcFile {
            nodes: map_into(value.nodes),
            messages: map_into(value.messages),
            signal_value_enums: enums,
        }
    }
}
