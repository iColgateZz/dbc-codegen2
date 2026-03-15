use crate::ir::signal_value_enum::SignalValueEnumIdx;

use super::transformation::TransformationNode;

/// Attach a SignalValueEnum to a Signal if there is one
/// matching the message ID and signal name
pub struct AttachSignalValueEnums;

impl TransformationNode for AttachSignalValueEnums {
    fn transform(&self, file: &mut crate::DbcFile) {
        //TODO: use hashmap for better performance
        for msg in &mut file.messages {
            for signal_idx in &msg.signal_idxs {
                let signal = &mut file.signals[signal_idx.0];

                for (idx, sve) in file.signal_value_enums.iter().enumerate() {
                    if sve.message_id == msg.id && sve.signal_name == signal.name.raw() {
                        signal.signal_value_enum = Some(SignalValueEnumIdx(idx));
                        break;
                    }
                }
            }
        }
    }
}
