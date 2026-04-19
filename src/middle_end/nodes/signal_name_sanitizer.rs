use std::collections::HashMap;

use super::transformation::TransformationNode;

/// Append {N} to signal name, ensure uniqueness within a message and correctness
pub struct SanitizeSignalNames;

impl TransformationNode for SanitizeSignalNames {
    fn transform(&self, file: &mut crate::DbcFile) {
        
        for msg in &mut file.messages {
            let mut counts: HashMap<String, usize> = HashMap::new();

            for signal_idx in &msg.signal_idxs {
                let signal = &mut file.signals[signal_idx.0];

                let base = signal.name.raw();
                let count = counts.entry(base.to_lowercase()).or_insert(0);

                let new_postfix = if *count == 0 {
                    "".into()
                } else {
                    format!("{}", count)
                };

                *count += 1;
                signal.name.postfix = new_postfix;
            }
        }

        for signal in &mut file.signals {
            signal.name.ensure_name_validity();
        }
    }
}