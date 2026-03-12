use super::helpers::ToUpperCamelCase;
use super::transformation_node::TransformationNode;

/// Sanitize the names of SignalValueEnum variants.
/// Remove the name of the signal and convert to
/// upper camelcase.
pub struct SanitizeSignalEnumVariantNames;

impl TransformationNode for SanitizeSignalEnumVariantNames {
    fn transform(&self, file: &mut crate::DbcFile) {
        for enum_def in &mut file.signal_value_enums {
            for variant in &mut enum_def.variants {
                variant.description = variant
                    .description
                    .replace(&enum_def.signal_name, "")
                    .to_upper_camelcase();
            }
        }
    }
}
