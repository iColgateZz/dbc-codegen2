use crate::middle_end::nodes::{CheckNode, Diagnostics};

pub struct CheckPipeline {
    nodes: Vec<Box<dyn CheckNode>>,
}

impl CheckPipeline {
    #[must_use]
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    #[must_use]
    pub fn add_node<N>(mut self, node: N) -> Self
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

impl Default for CheckPipeline {
    fn default() -> Self {
        Self::new()
    }
}

//TODO: checker node ideas
// enum values fit raw range
// mux value fits raw range
// if extended value type is f32 or f64 then signals has to be of that size
// AttachSignalValueEnumType overwrites sve physical type. Ensure that all signals using one enum have same physical type?
