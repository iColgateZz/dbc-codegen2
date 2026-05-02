use std::collections::HashMap;

use super::transformation::TransformationNode;
use crate::ir::signal_value_enum::SignalValueEnum;

/// Deduplicate SignalValueEnums.
///
/// If enabled, performs the deduplication.
/// Enums with same names and variants are treated as one.
pub struct DeduplicateSignalValueEnums {
    pub dedup_enabled: bool,
}

#[derive(Hash, PartialEq, Eq)]
struct EnumSignature {
    name: String,
    variants: Vec<(String, i64)>,
}

impl EnumSignature {
    fn from_enum(sve: &SignalValueEnum) -> Self {
        Self {
            name: sve.name.raw().into(),
            variants: sve
                .variants
                .iter()
                .map(|v| (v.description.clone(), v.value))
                .collect(),
        }
    }
}

impl TransformationNode for DeduplicateSignalValueEnums {
    fn transform(&self, file: &mut crate::DbcFile) {
        if !self.dedup_enabled {
            return;
        }

        let mut map: HashMap<EnumSignature, usize> = HashMap::new();
        let mut new_enums: Vec<SignalValueEnum> = Vec::new();
        let mut remap: Vec<usize> = vec![0; file.signal_value_enums.len()];

        for (old_idx, sve) in file.signal_value_enums.iter().enumerate() {
            let sig = EnumSignature::from_enum(sve);

            let new_idx = match map.get(&sig) {
                Some(&idx) => idx,
                None => {
                    let idx = new_enums.len();
                    new_enums.push(sve.clone());
                    map.insert(sig, idx);
                    idx
                }
            };

            remap[old_idx] = new_idx;
        }

        for sig in &mut file.signals {
            if let Some(idx) = &mut sig.signal_value_enum_idx {
                idx.0 = remap[idx.0];
            }
        }

        file.signal_value_enums = new_enums;
    }
}
