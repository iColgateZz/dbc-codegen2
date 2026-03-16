use crate::utils::infer_repr_type;

use super::transformation::TransformationNode;

/// Infer needed type for a SingalValueEnum
pub struct InferSignalValueEnumType;

impl TransformationNode for InferSignalValueEnumType {
    fn transform(&self, file: &mut crate::DbcFile) {
        for sig in &mut file.signals {
            if let Some(sve) = &mut sig.signal_value_enum {
                let values = sve.variants.iter().map(|v| v.value);
                sve.repr_type = infer_repr_type(values);
            }
        }
    }
}