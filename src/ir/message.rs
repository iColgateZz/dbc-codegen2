use crate::ir::{Identifier, SignalIdx};
use can_dbc::MessageId as ParsedMessageId;
use can_dbc::Transmitter as ParsedTransmitter;

#[derive(Debug, Clone)]
pub struct Message {
    pub id: MessageId,
    pub name: Identifier,
    pub size: u64,
    pub transmitter: Transmitter,
    pub signal_idxs: Vec<SignalIdx>,
}

impl Message {
    pub fn from_parsed(
        id: ParsedMessageId, 
        name: String,
        size: u64,
        transmitter: ParsedTransmitter,
        signals: Vec<SignalIdx>
    ) -> Self {
        Message {
            id: id.into(),
            name: Identifier(name),
            size: size,
            transmitter: Transmitter::from(transmitter),
            signal_idxs: signals,
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
