use crate::middle_end::nodes::transformation::TransformationNode;

pub struct TransformationPipeline {
    nodes: Vec<Box<dyn TransformationNode>>,
}

impl TransformationPipeline {
    #[must_use]
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    #[must_use]
    pub fn add_node<N>(mut self, node: N) -> Self
    where
        N: TransformationNode + 'static,
    {
        self.nodes.push(Box::new(node));
        self
    }

    pub fn run(self, file: &mut crate::DbcFile) {
        for node in self.nodes {
            node.transform(file);
        }
    }
}

impl Default for TransformationPipeline {
    fn default() -> Self {
        Self::new()
    }
}
