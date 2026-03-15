use super::transformation::TransformationNode;

/// Filter out messages that are not relevant
pub struct FilterRelevantMessages;

impl TransformationNode for FilterRelevantMessages {
    fn transform(&self, file: &mut crate::DbcFile) {
        file.messages
            .retain(|msg| msg.name.raw() != "VECTOR__INDEPENDENT_SIG_MSG");
    }
}
