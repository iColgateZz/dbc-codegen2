use std::collections::HashMap;

use super::transformation::TransformationNode;

/// Append _ENUM{N} to sve name, ensure uniqueness and correctness
pub struct SanitizeSVENames;

impl TransformationNode for SanitizeSVENames {
    fn transform(&self, file: &mut crate::DbcFile) {
        let mut counts: HashMap<String, usize> = HashMap::new();

        for sve in &mut file.signal_value_enums {
            let base = format!("{}_ENUM", sve.name.raw);
            let count = counts.entry(base.to_lowercase()).or_insert(0);

            let new_postfix = if *count == 0 {
                "_ENUM".into()
            } else {
                format!("_ENUM{}", count)
            };

            *count += 1;
            sve.name.postfix = new_postfix;
        }

        for sve in &mut file.signal_value_enums {
            sve.name.ensure_name_validity();
        }
    }
}