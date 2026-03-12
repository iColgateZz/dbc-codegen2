use can_dbc::Node as ParsedNode;

#[derive(Debug, Clone)]
pub struct NodeName(pub String);

#[derive(Debug, Clone)]
pub struct Node {
    pub name: NodeName,
}
impl From<ParsedNode> for Node {
    fn from(value: ParsedNode) -> Self {
        Node {
            name: (NodeName(value.0)),
        }
    }
}