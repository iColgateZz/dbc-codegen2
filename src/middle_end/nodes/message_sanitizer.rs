use super::helpers::ToUpperCamelCase;
use super::transformation_node::TransformationNode;

/// Sanitize the names of Message structs.
pub struct SanitizeMessageNames;

impl TransformationNode for SanitizeMessageNames {
    fn transform(&self, file: &mut crate::DbcFile) {
        for msg in &mut file.messages {
            msg.name.0 = msg.original_name.0.to_upper_camelcase();
        }
    }
}
