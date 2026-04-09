use super::transformation::TransformationNode;

/// Attach physical type to SignalValueEnum.
pub struct AttachSignalValueEnumType;

impl TransformationNode for AttachSignalValueEnumType {
    fn transform(&self, file: &mut crate::DbcFile) {
        for sig in &mut file.signals {
            if let Some(sve_idx) = sig.signal_value_enum_idx {
                let sve = &mut file.signal_value_enums[sve_idx.0];

                sve.phys_type = sig.physical_type;
            }
        }
    }
}
