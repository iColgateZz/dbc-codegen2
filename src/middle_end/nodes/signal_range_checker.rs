use crate::{
    ir::{
        signal_extended_value_type::ExtendedValueType,
        signal_layout::{SignalLayout, ValueType},
    },
    middle_end::nodes::{CheckNode, Diagnostics},
};

pub struct CheckSignalPhysicalRangeRepresentable {
    pub zero_zero_range_allows_all: bool,
}

impl CheckNode for CheckSignalPhysicalRangeRepresentable {
    fn check(&self, file: &crate::DbcFile, diagnostics: &mut Diagnostics) {
        for msg in &file.messages {
            for sig_idx in &msg.signal_idxs {
                let sig = &file.signals[sig_idx.0];
                let layout = &file.signal_layouts[sig.layout.0];

                //TODO: checking does not make much sense when range is [0|0].
                //      use inferred type bounds instead?
                if self.zero_zero_range_allows_all && layout.min == 0.0 && layout.max == 0.0 {
                    continue;
                }

                //ignore floats and doubles
                let Some((scaled_min, scaled_max)) =
                    scaled_raw_range(layout, sig.extended_type.clone())
                else {
                    continue;
                };

                let eps = range_epsilon([scaled_min, scaled_max, layout.min, layout.max]);

                if layout.min < scaled_min - eps || layout.max > scaled_max + eps {
                    diagnostics.error(format!(
                        "Signal '{}' in message '{}' declares physical range [{}|{}], \
                         but raw range scaled by factor/offset can only represent [{}|{}] \
                         (size={} bits, type={}, factor={}, offset={})",
                        sig.name.raw(),
                        msg.name.raw(),
                        layout.min,
                        layout.max,
                        scaled_min,
                        scaled_max,
                        layout.size,
                        value_type_name(layout.value_type),
                        layout.factor,
                        layout.offset,
                    ));
                }
            }
        }
    }
}

fn scaled_raw_range(layout: &SignalLayout, extended_type: ExtendedValueType) -> Option<(f64, f64)> {
    if layout.size == 0 {
        return None;
    }

    let (raw_min, raw_max) = match extended_type {
        ExtendedValueType::Integer => integer_raw_range(layout)?,

        // SIG_VALTYPE_ float/double is not checked here.
        ExtendedValueType::Float32 | ExtendedValueType::Double64 => return None,
    };

    let a = raw_min * layout.factor + layout.offset;
    let b = raw_max * layout.factor + layout.offset;

    Some((a.min(b), a.max(b)))
}

fn integer_raw_range(layout: &SignalLayout) -> Option<(f64, f64)> {
    let size = layout.size;

    match layout.value_type {
        ValueType::Signed => {
            let magnitude = pow2(size.checked_sub(1)?)?;
            Some((-magnitude, magnitude - 1.0))
        }
        ValueType::Unsigned => Some((0.0, pow2(size)? - 1.0)),
    }
}

fn pow2(bits: u64) -> Option<f64> {
    if bits > 1023 {
        None
    } else {
        Some(2f64.powi(bits as i32))
    }
}

fn range_epsilon(values: impl IntoIterator<Item = f64>) -> f64 {
    let magnitude = values.into_iter().map(f64::abs).fold(1.0, f64::max);

    magnitude * 1e-9
}

fn value_type_name(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::Signed => "signed",
        ValueType::Unsigned => "unsigned",
    }
}
