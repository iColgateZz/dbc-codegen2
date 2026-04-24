use crate::{
    ir::signal::MultiplexIndicator,
    middle_end::nodes::{CheckNode, Diagnostics},
};

pub struct CheckUnsupportedMultiplexing;

impl CheckNode for CheckUnsupportedMultiplexing {
    fn check(&self, file: &crate::DbcFile, diagnostics: &mut Diagnostics) {
        for msg in &file.messages {
            let mut multiplexors = Vec::new();

            for sig_idx in &msg.signal_idxs {
                let sig = &file.signals[sig_idx.0];

                match sig.multiplexer {
                    MultiplexIndicator::Multiplexor => {
                        multiplexors.push(sig.name.raw());
                    }

                    MultiplexIndicator::MultiplexorAndMultiplexedSignal(mux_value) => {
                        diagnostics.error(format!(
                            "Signal '{}' in message '{}' is both a multiplexor and multiplexed signal with mux value {}. This feature is not supported.",
                            sig.name.raw(),
                            msg.name.raw(),
                            mux_value,
                        ));

                        multiplexors.push(sig.name.raw());
                    }

                    MultiplexIndicator::Plain
                    | MultiplexIndicator::MultiplexedSignal(_) => {}
                }
            }

            if multiplexors.len() > 1 {
                diagnostics.error(format!(
                    "Message '{}' has multiple multiplexors: {}. Only one multiplexor per message is supported.",
                    msg.name.raw(),
                    multiplexors
                        .iter()
                        .map(|name| format!("'{name}'"))
                        .collect::<Vec<_>>()
                        .join(", "),
                ));
            }

        }

        if file.has_extended_mux_symbols {
            diagnostics.error("File contains extended multiplex symbols. This feature is not supported.");
        }
    }
}