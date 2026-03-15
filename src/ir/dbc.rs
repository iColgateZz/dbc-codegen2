use crate::ir::{Message, Node, Signal, SignalIdx, SignalValueEnum, helpers::map_into};
use can_dbc::Dbc as ParsedDbc;
use can_dbc::Message as ParsedMessage;
use can_dbc::ValueDescription as ParsedValueDescription;

#[derive(Debug, Default)]
pub struct DbcFile {
    pub nodes: Vec<Node>,
    pub messages: Vec<Message>,
    pub signals: Vec<Signal>,
    pub signal_value_enums: Vec<SignalValueEnum>,
    //TODO: store signal_extended_value_type_list from can_dbc
    //      for signal type inference

    //TODO: consider how to use can_dbc::value_tables. Basically,
    //      these are global enums for signal values

    //TODO: can_dbc::comments and attribute_* stuff may be
    //      used as metadata in generated code

    //TODO: consider how to use can_dbc::signal_types and 
    //      signal_type_refs. original dbc-codegen does not
    //      support them. They allow to define a signal once
    //      and then reuse them later.

    //TODO: can_dbc::extended_multiplex is probably also needed
}

impl From<ParsedDbc> for DbcFile {
    fn from(value: ParsedDbc) -> Self {
        let mut file = DbcFile::default();

        file.nodes = map_into(value.nodes);

        for value_enum in value.value_descriptions {
            match value_enum {
                ParsedValueDescription::Signal { 
                    message_id, 
                    name, 
                    value_descriptions 
                } => {
                    let sve = SignalValueEnum::from_parsed(message_id, name, value_descriptions);
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
