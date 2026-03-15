use crate::ir::map_into;
use crate::ir::{Identifier, Signal};
use can_dbc::Message as ParsedMessage;
use can_dbc::MessageId as ParsedMessageId;
use can_dbc::Transmitter as ParsedTransmitter;

#[derive(Debug, Clone)]
pub struct Message {
    pub id: MessageId,
    pub name: Identifier,
    pub size: u64,
    pub transmitter: Transmitter,
    pub signals: Vec<Signal>,
}
impl From<ParsedMessage> for Message {
    fn from(value: ParsedMessage) -> Self {
        Message {
            id: MessageId::from(value.id),
            name: Identifier(value.name),
            size: value.size,
            transmitter: Transmitter::from(value.transmitter),
            signals: map_into(value.signals),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MessageId {
    Standard(u16),
    Extended(u32),
}
impl From<ParsedMessageId> for MessageId {
    fn from(value: ParsedMessageId) -> Self {
        match value {
            ParsedMessageId::Standard(v) => MessageId::Standard(v),
            ParsedMessageId::Extended(v) => MessageId::Extended(v),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Transmitter {
    Node(Identifier),
    VectorXXX,
}
impl From<ParsedTransmitter> for Transmitter {
    fn from(value: ParsedTransmitter) -> Self {
        match value {
            ParsedTransmitter::NodeName(s) => Transmitter::Node(Identifier(s)),
            ParsedTransmitter::VectorXXX => Transmitter::VectorXXX,
        }
    }
}
