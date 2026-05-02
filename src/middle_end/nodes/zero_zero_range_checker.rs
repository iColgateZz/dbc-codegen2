use crate::middle_end::nodes::CheckNode;

pub struct CheckZeroZeroRanges {
    pub zero_zero_range_allows_all: bool,
}

impl CheckNode for CheckZeroZeroRanges {
    fn check(&self, file: &crate::DbcFile, diagnostics: &mut super::Diagnostics) {
        if self.zero_zero_range_allows_all {
            return;
        }

        for sig in &file.signals {
            let layout = &file.signal_layouts[sig.layout.0];

            if layout.min == layout.max && layout.max == 0.0 {
                let name = sig.name.raw();
                let msg = format!(
                    "Signal '{name}' has [0|0] range. This usually means the vendor did not specify limits. Consider using '--zero-zero-range-allows-all' flag"
                );
                diagnostics.warning(msg);
            }
        }
    }
}
