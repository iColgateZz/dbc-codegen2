use crate::ir::DbcFile;

pub trait TransformationNode {
    fn transform(&self, file: &mut DbcFile);
}
