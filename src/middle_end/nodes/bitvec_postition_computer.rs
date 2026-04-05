use super::transformation::TransformationNode;
use crate::ir::signal_layout::ByteOrder;

/// Pre-compute bitvec slice positions for each signal layout.
/// Translates DBC start_bit (which varies by byte order) into
/// a unified [start..end] range suitable for bitvec indexing.
pub struct ComputeBitvecPositions;

impl TransformationNode for ComputeBitvecPositions {
    fn transform(&self, file: &mut crate::DbcFile) {
        for layout in &mut file.signal_layouts {
            let (start, end) = match layout.byte_order {
                ByteOrder::LittleEndian => {
                    let start = layout.start_bit as usize;
                    let end = start + layout.size as usize;
                    (start, end)
                }
                ByteOrder::BigEndian => {
                    let start_bit = layout.start_bit;
                    let x = (start_bit / 8) * 8;
                    let y = 7 - (start_bit % 8);
                    let start = (x + y) as usize;
                    let end = start + layout.size as usize;
                    (start, end)
                }
            };

            layout.bitvec_start = start;
            layout.bitvec_end = end;
        }
    }
}
