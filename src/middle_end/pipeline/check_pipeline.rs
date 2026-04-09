use crate::middle_end::nodes::{CheckNode, Diagnostics};

pub struct CheckPipeline {
    nodes: Vec<Box<dyn CheckNode>>,
}

impl CheckPipeline {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn add<N>(mut self, node: N) -> Self
    where
        N: CheckNode + 'static,
    {
        self.nodes.push(Box::new(node));
        self
    }

    pub fn run(self, file: &crate::DbcFile, diagnostics: &mut Diagnostics) {
        for node in self.nodes {
            node.check(file, diagnostics);
        }
    }
}
