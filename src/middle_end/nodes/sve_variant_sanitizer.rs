use heck::ToUpperCamelCase;

use super::transformation::TransformationNode;
use crate::ir::identifier::is_valid_identifier;

/// Sanitize the names of SignalValueEnum variants.
/// Remove the name of the signal and convert to upper camel case.
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
            }
        }
    }
}