
use crate::utils::infer_repr_type;

use super::transformation::TransformationNode;

/// Infer needed type for a SingalValueEnum
pub struct InferSignalValueEnumType;

impl TransformationNode for InferSignalValueEnumType {
    fn transform(&self, file: &mut crate::DbcFile) {
        for sve in &mut file.signal_value_enums {
            let values = sve.variants.iter().map(|v| file.value_descriptions[v.0].value);
            sve.repr_type = infer_repr_type(values);
        }
    }
}