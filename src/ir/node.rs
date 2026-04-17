use can_dbc::Node as ParsedNode;

use crate::ir::identifier::Identifier;

#[derive(Debug, Clone)]
pub struct Node {
    pub name: Identifier,
}
impl From<ParsedNode> for Node {
    fn from(value: ParsedNode) -> Self {
        Node {
            name: (Identifier::from_raw(value.0)),
        }
    }
}
