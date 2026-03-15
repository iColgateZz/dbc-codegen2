use crate::utils::ToUpperCamelCase;
use super::transformation::TransformationNode;

/// Sanitize the names of SignalValueEnum variants.
/// Remove the name of the signal and convert to
/// upper camelcase.
pub struct SanitizeSignalEnumVariantNames;

impl TransformationNode for SanitizeSignalEnumVariantNames {
    fn transform(&self, file: &mut crate::DbcFile) {
        for sve in &mut file.signal_value_enums {
            for variant in &mut sve.variants {
                variant.description = variant
                    .description
                    .replace(&sve.signal_name, "")
                    .to_upper_camelcase();
            }
        }
    }
}
