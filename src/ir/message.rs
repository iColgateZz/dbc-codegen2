use crate::ir::signal::{MultiplexIndicator, Signal};
use crate::ir::{Identifier, MessageLayoutIdx, SignalIdx};
use can_dbc::MessageId as ParsedMessageId;
use can_dbc::Transmitter as ParsedTransmitter;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Message {
    pub id: MessageId,
    pub name: Identifier,
    pub size: u64,
    pub transmitter: Transmitter,
    pub signal_idxs: Vec<SignalIdx>,
    pub layout: MessageLayoutIdx,
    pub comment: Option<String>,
}

impl Message {
    pub fn from_parsed(
        id: ParsedMessageId,
        name: String,
        size: u64,
        transmitter: ParsedTransmitter,
        signals: Vec<SignalIdx>,
        layout: MessageLayoutIdx,
        comment: Option<String>,
    ) -> Self {
        Message {
            id: id.into(),
            name: Identifier::from_raw(name),
            size: size,
            transmitter: Transmitter::from(transmitter),
            signal_idxs: signals,
            layout: layout,
            comment
        }
    }

    pub fn classify_signals(&self, signals: &[Signal]) -> MessageSignalClassification {
        let mut plain = Vec::new();
        let mut mux_signal = None;
        let mut muxed: BTreeMap<u64, Vec<SignalIdx>> = BTreeMap::new();

        for &idx in &self.signal_idxs {
            match &signals[idx.0].multiplexer {
                MultiplexIndicator::Plain => plain.push(idx),
                MultiplexIndicator::Multiplexor => mux_signal = Some(idx),
                MultiplexIndicator::MultiplexedSignal(v) => {
                    muxed.entry(*v).or_default().push(idx);
                }

                // intentionally skip
                MultiplexIndicator::MultiplexorAndMultiplexedSignal(_) => {}
            }
        }

        if muxed.is_empty() {
            // If there's a mux selector but no muxed groups, treat it as plain
            if let Some(m) = mux_signal {
                plain.push(m);
            }
            MessageSignalClassification::Plain { signals: plain }
        } else {
            MessageSignalClassification::Multiplexed {
                plain,
                mux_signal: mux_signal
                    .expect("message has muxed signals but no multiplexor signal"),
                muxed,
            }
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
            ParsedTransmitter::NodeName(s) => Transmitter::Node(Identifier::from_raw(s)),
            ParsedTransmitter::VectorXXX => Transmitter::VectorXXX,
        }
    }
}

pub enum MessageSignalClassification {
    Plain {
        signals: Vec<SignalIdx>,
    },
    Multiplexed {
        plain: Vec<SignalIdx>,
        mux_signal: SignalIdx,
        muxed: BTreeMap<u64, Vec<SignalIdx>>,
    },
}
