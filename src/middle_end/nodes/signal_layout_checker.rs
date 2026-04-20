use crate::middle_end::nodes::{CheckNode, Diagnostics};

pub struct CheckSignalLayoutValidity;

impl CheckNode for CheckSignalLayoutValidity {
    fn check(&self, file: &crate::DbcFile, diagnostics: &mut Diagnostics) {
        for msg in &file.messages {
            let msg_bits = msg.size.saturating_mul(8);

            for sig_idx in &msg.signal_idxs {
                let sig = &file.signals[sig_idx.0];
                let layout = &file.signal_layouts[sig.layout.0];

                if msg_bits == 0 && layout.size > 0 {
                    diagnostics.error(format!(
                        "Message '{}' has size 0, but contains signal '{}' with size {}",
                        msg.name.raw(),
                        sig.name.raw(),
                        layout.size
                    ));
                }

                if layout.size == 0 {
                    diagnostics.error(format!(
                        "Signal '{}' in message '{}' has size 0",
                        sig.name.raw(),
                        msg.name.raw(),
                    ));
                }

                let max_start_bit = msg_bits - 1;
                if layout.start_bit > max_start_bit {
                    diagnostics.error(format!(
                        "Signal '{}' in message '{}' has start_bit={} but valid range is 0..={}",
                        sig.name.raw(),
                        msg.name.raw(),
                        layout.start_bit,
                        max_start_bit
                    ));
                }

                if layout.factor == 0.0 {
                    diagnostics.error(format!(
                        "Signal '{}' in message '{}' has factor 0",
                        sig.name.raw(),
                        msg.name.raw()
                    ));
                }

                if layout.min > layout.max {
                    diagnostics.error(format!(
                        "Signal '{}' in message '{}' has invalid range [{}|{}]: min > max",
                        sig.name.raw(),
                        msg.name.raw(),
                        layout.min,
                        layout.max
                    ));
                }
            }
        }
    }
}
