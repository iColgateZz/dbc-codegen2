use std::collections::HashMap;

use super::transformation::TransformationNode;

/// Append _ENUM to SVE name and ensure uniqueness
pub struct SanitizeSVENames;

impl TransformationNode for SanitizeSVENames {
    fn transform(&self, file: &mut crate::DbcFile) {
        let mut counts: HashMap<String, usize> = HashMap::new();

        for sve in &mut file.signal_value_enums {
            let base = format!("{}_ENUM", sve.name.raw);

            let count = counts.entry(base.to_lowercase()).or_insert(0);

            let new_name = if *count == 0 {
                base
            } else {
                format!("{}{}", base, count)
            };

            *count += 1;

            sve.name.raw = new_name;
        }
    }
}