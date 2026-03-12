use crate::nodes::transformation_node::TransformationNode;

/// Attach a SignalValueEnum to a Signal if there is one
/// matching the message ID and signal name
pub struct SignalValueEnumAttacher;

impl TransformationNode for SignalValueEnumAttacher {
    fn transform(&self, file: &mut crate::DbcFile) {
        //TODO: use hashmap for better performance
        for msg in &mut file.messages {
            for signal in &mut msg.signals {
                signal.signal_value_enum = file
                    .signal_value_enums
                    .iter()
                    .find(|v| {
                        v.message_id == msg.id
                            && v.signal_name == signal.original_name.0
                    })
                    .cloned();
            }
        }
    }
}