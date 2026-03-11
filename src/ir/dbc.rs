use std::collections::HashMap;

use crate::ir::{Message, MessageId, Node, ToUpperCamelCase, ValueDescription, map_into};
use can_dbc::Dbc as ParsedDbc;

#[derive(Debug, Clone)]
pub struct DbcFile {
    pub nodes: Vec<Node>,
    pub messages: Vec<Message>,
}
impl DbcFile {
    pub fn from_dbc(dbc: ParsedDbc) -> Self {
        let mut value_desc_map: HashMap<(MessageId, String), Vec<ValueDescription>> = dbc
            .messages
            .iter()
            .flat_map(|msg| {
                msg.signals.iter().filter_map(|sig| {
                    let descs = dbc.value_descriptions_for_signal(msg.id, &sig.name)?;
                    let converted = descs
                        .iter()
                        .map(|vd| ValueDescription {
                            value: vd.id,
                            description: vd
                                .description
                                .clone()
                                .replace(&sig.name, "")
                                .to_upper_camelcase(),
                        })
                        .collect();
                    Some(((MessageId::from(msg.id), sig.name.clone()), converted))
                })
            })
            .collect();

        let mut file = DbcFile::from(dbc);

        for message in &mut file.messages {
            for signal in &mut message.signals {
                let key = (message.id.clone(), signal.original_name.0.clone());
                if let Some(descs) = value_desc_map.remove(&key) {
                    signal.value_descriptions = descs;
                }
            }
        }

        file
    }
}
impl From<ParsedDbc> for DbcFile {
    fn from(value: ParsedDbc) -> Self {
        DbcFile {
            nodes: map_into(value.nodes),
            messages: map_into(value.messages),
        }
    }
}
