use std::collections::HashSet;

use crate::middle_end::nodes::{CheckNode, Diagnostics};

pub struct CheckEnumVariants;

impl CheckNode for CheckEnumVariants {
    fn check(&self, file: &crate::DbcFile, diagnostics: &mut Diagnostics) {
        for signal in &file.signals {
            if let Some(idx) = signal.signal_value_enum_idx {
                let sve = &file.signal_value_enums[idx.0];

                if sve.variants.is_empty() {
                    diagnostics.error(format!(
                        "Enum '{}' used by signal '{}' is empty",
                        sve.name.raw(),
                        signal.name.raw(),
                    ));
                }

                let mut set = HashSet::new();
                for variant in &sve.variants {
                    let new = set.insert(variant.value);

                    if !new {
                        diagnostics.error(format!(
                            "Enum '{}' used by signal '{}' has duplicated variant value '{}'",
                            sve.name.raw(),
                            signal.name.raw(),
                            variant.value,
                        ));
                    }
                }
            }
        }
    }
}
