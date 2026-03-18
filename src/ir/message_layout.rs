use crate::ir::SignalLayoutIdx;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MessageLayoutIdx(pub usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageLayout {
    pub signal_layouts: Vec<SignalLayoutIdx>,
}