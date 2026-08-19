use super::transformation::TransformationNode;
use crate::ir::signal_layout::ByteOrder;

/// Pre-compute bitvec slice positions for each signal layout.
/// Translates DBC `start_bit` (which varies by byte order) into
/// a unified [start..end] range suitable for bitvec indexing.
pub struct ComputeBitvecPositions;

impl TransformationNode for ComputeBitvecPositions {
    fn transform(&self, file: &mut crate::DbcFile) {
        for layout in &mut file.signal_layouts {
            let (start, size) = (
                usize::try_from(layout.start_bit).expect("signal start_bit exceeded usize::MAX"),
                usize::try_from(layout.size).expect("signal size exceeded usize::MAX"),
            );

            let start_adjusted = match layout.byte_order {
                ByteOrder::LittleEndian => start,
                // This ensures correct start-end bit positions for the
                // Motorola BigEndian "ZigZag" encoding when paired with
                // bitvec's `*<Msb0>::*_be(...)` functions.
                ByteOrder::BigEndian => start ^ 7,
            };

            layout.bitvec_start = start_adjusted;
            layout.bitvec_end = start_adjusted + size;
        }
    }
}
