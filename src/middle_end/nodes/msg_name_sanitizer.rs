use std::collections::HashMap;

use super::transformation::TransformationNode;

/// Append _MSG to message name and ensure uniqueness
pub struct SanitizeMessageNames;

impl TransformationNode for SanitizeMessageNames {
    fn transform(&self, file: &mut crate::DbcFile) {
        let mut counts: HashMap<String, usize> = HashMap::new();

        for msg in &mut file.messages {
            let base = format!("{}_MSG", msg.name.raw);

            let count = counts.entry(base.to_lowercase()).or_insert(0);

            let new_postfix = if *count == 0 {
                "_MSG".into()
            } else {
                format!("_MSG{}", count)
            };

            *count += 1;

            msg.name.postfix = new_postfix;
        }
    }
}