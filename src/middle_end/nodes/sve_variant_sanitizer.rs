use heck::ToUpperCamelCase;
use std::collections::HashMap;

use super::transformation::TransformationNode;
use crate::ir::identifier::is_valid_identifier;

/// Sanitize the names of SignalValueEnum variants.
/// Remove the name of the signal and convert to upper camel case.
/// If duplicates appear after sanitization, append the numeric value.
pub struct SanitizeSignalEnumVariantNames;

impl TransformationNode for SanitizeSignalEnumVariantNames {
    fn transform(&self, file: &mut crate::DbcFile) {
        for sig in &file.signals {
            if let Some(sve_idx) = sig.signal_value_enum_idx {
                let sve = &mut file.signal_value_enums[sve_idx.0];

                for variant in &mut sve.variants {
                    let mut name = variant.description.replace(sig.name.raw(), "");
                    name = name.to_upper_camel_case();

                    if name.is_empty() {
                        name = format!("V{}", variant.value);
                    }

                    if !is_valid_identifier(&name) {
                        name = format!("V{name}");
                    }

                    variant.description = name;
                }

                let mut counts: HashMap<String, usize> = HashMap::new();
                for variant in &sve.variants {
                    *counts.entry(variant.description.clone()).or_insert(0) += 1;
                }

                for variant in &mut sve.variants {
                    if let Some(count) = counts.get(&variant.description) {
                        if *count > 1 {
                            variant.description =
                                format!("{}{}", variant.description, variant.value);
                        }
                    }
                }
            }
        }
    }
}
