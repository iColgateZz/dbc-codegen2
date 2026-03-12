use crate::middle_end::nodes::helpers::ToUpperCamelCase;
use super::transformation::TransformationNode;

/// Sanitize the names of Signals inside the SignalValueEnum structs.
pub struct SanitizeSignalEnumSignalNames;

impl TransformationNode for SanitizeSignalEnumSignalNames {
    fn transform(&self, file: &mut crate::DbcFile) {
        for msg in &mut file.messages {
            for sig in &mut msg.signals {
                if let Some(enum_def) = &mut sig.signal_value_enum {
                    enum_def.signal_name = enum_def.signal_name.to_upper_camelcase();
                }
            }
        }
    }
}