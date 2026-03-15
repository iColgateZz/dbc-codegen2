use crate::ir::value_description::ValueDescriptionIdx;
use crate::ir::{Message, Node, Signal, SignalIdx, SignalValueEnum, ValueDescription, helpers::map_into};
use can_dbc::Dbc as ParsedDbc;
use can_dbc::Message as ParsedMessage;
use can_dbc::ValueDescription as ParsedValueDescription;

#[derive(Debug, Default)]
pub struct DbcFile {
    pub nodes: Vec<Node>,
    pub messages: Vec<Message>,
    pub signals: Vec<Signal>,
    pub signal_value_enums: Vec<SignalValueEnum>,
    pub value_descriptions: Vec<ValueDescription>,
}

impl From<ParsedDbc> for DbcFile {
    fn from(value: ParsedDbc) -> Self {
        let mut file = DbcFile::default();

        file.nodes = map_into(value.nodes);

        for e in value.value_descriptions {
            match e {
                ParsedValueDescription::Signal { 
                    message_id, 
                    name, 
                    value_descriptions 
                } => {
                    let mut description_ids = vec![];
                    for desc in value_descriptions {
                        let id = file.value_descriptions.len();
                        file.value_descriptions.push(ValueDescription::from(desc));
                        description_ids.push(ValueDescriptionIdx(id));
                    }

                    let sve = SignalValueEnum::from_parsed(message_id, name, description_ids);
                    file.signal_value_enums.push(sve);
                },
                _ => (),
            };
        }


        for msg in value.messages {
            let mut signal_ids = vec![];
            for sig in msg.signals {
                let id = file.signals.len();
                file.signals.push(Signal::from(sig));
                signal_ids.push(SignalIdx(id));
            }
            
            let ParsedMessage { id, name, size, transmitter, .. } = msg;
            file.messages.push(Message::from_parsed(id, name, size, transmitter, signal_ids));
        }

        file
    }
}
