use super::transformation::TransformationNode;

/// Prefix SignalValueEnum with Message name.
/// 
/// Performs the task when enum deduplication is disabled.
pub struct PrefixSignalValueEnumName {
    pub dedup_enabled: bool,
}

impl TransformationNode for PrefixSignalValueEnumName {
    fn transform(&self, file: &mut crate::DbcFile) {
        if self.dedup_enabled {
            return;
        }
        
        for msg in &mut file.messages {
            for sig_idx in &mut msg.signal_idxs {
                let sig = &mut file.signals[sig_idx.0];

                if let Some(sve_idx) = sig.signal_value_enum_idx {
                    let sve = &mut file.signal_value_enums[sve_idx.0];

                    sve.name = format!("{}{}", msg.name.raw, sve.name);
                }
            }
        }
    }
}
