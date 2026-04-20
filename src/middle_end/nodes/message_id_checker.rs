use std::collections::HashMap;

use crate::{
    ir::message::{MessageId, Message},
    middle_end::nodes::{CheckNode, Diagnostics},
};

pub struct CheckUniqueMessageIds;

impl CheckNode for CheckUniqueMessageIds {
    fn check(&self, file: &crate::DbcFile, diagnostics: &mut Diagnostics) {
        let mut ids: HashMap<&MessageId, Vec<&Message>> = HashMap::new();

        for msg in &file.messages {
            ids.entry(&msg.id).or_default().push(msg);
        }

        for (id, msgs) in ids {
            if msgs.len() > 1 {
                let names = msgs
                    .iter()
                    .map(|m| format!("'{}'", m.name.raw()))
                    .collect::<Vec<_>>()
                    .join(", ");

                diagnostics.error(format!(
                    "Duplicate message id {:?} used by messages: {}",
                    id, names
                ));
            }
        }
    }
}
