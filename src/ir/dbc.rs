use crate::ir::{Message, Node, Signal, SignalLayout, message_layout::MessageLayout};

#[derive(Debug, Default)]
pub struct DbcFile {
    pub nodes: Vec<Node>,
    pub messages: Vec<Message>,
    pub message_layouts: Vec<MessageLayout>,
    pub signals: Vec<Signal>,
    pub signal_layouts: Vec<SignalLayout>
    //TODO: split signal into SignalLayout and SignalInstance
    //      All core fields -> Layout
    //      Instance has layout idx, name, receivers
    //      Messages hold SignalInstanceIdx, also add MessageLayout
    //      Later, in one of the passes all MessageLayouts are determined
    //      Something like this should work

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
