use super::transformation::TransformationNode;

/// Append _MSG to message name
pub struct SanitizeMessageNames;

impl TransformationNode for SanitizeMessageNames {
    fn transform(&self, file: &mut crate::DbcFile) {
        for msg in &mut file.messages {
            msg.name.0 = format!("{}_MSG", msg.name.0);
        }
    }
}
