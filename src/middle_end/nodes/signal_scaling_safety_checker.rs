use crate::{
    ir::{
        signal_layout::{SignalLayout, ValueType},
        signal_value_type::{IntReprType, PhysicalType, RawType},
    },
    middle_end::nodes::{CheckNode, Diagnostics},
};

/// Check that generated integer scaling arithmetic is safe for the inferred
/// raw and physical types.
///
/// 1. Decode/getter path:
///    raw -> raw * factor -> raw * factor + offset
///    The intermediate product and final physical value must fit the inferred
///    physical integer type.
///
/// 2. Encode/setter path, when the DBC declares a non-[0|0] physical range:
///    physical -> physical - offset -> (physical - offset) / factor
///    The intermediate subtraction must fit the inferred physical integer type,
///    and the resulting raw values required by the declared physical range must
///    fit the actual raw bit-domain.
///
/// Floating-point signals are not checked here.
pub struct CheckSignalScalingArithmeticSafety;

impl CheckNode for CheckSignalScalingArithmeticSafety {
    fn check(&self, file: &crate::DbcFile, diagnostics: &mut Diagnostics) {
        for msg in &file.messages {
            for sig_idx in &msg.signal_idxs {
                let sig = &file.signals[sig_idx.0];
                let layout = &file.signal_layouts[sig.layout.0];

                let RawType::Integer(raw_repr) = sig.raw_type else {
                    continue;
                };

                let Some(phys_repr) = integer_physical_repr(sig.physical_type) else {
                    continue;
                };

                if !is_integer(layout.factor) || !is_integer(layout.offset) {
                    continue;
                }

                let factor = layout.factor as i128;
                let offset = layout.offset as i128;

                if factor == 0 {
                    continue;
                }

                let raw_domain = actual_raw_domain(layout);
                let phys_type_domain = int_repr_domain(phys_repr);
                let raw_type_domain = int_repr_domain(raw_repr);

                check_getter_path(
                    sig.name.raw(),
                    msg.name.raw(),
                    layout,
                    factor,
                    offset,
                    raw_domain,
                    phys_type_domain,
                    diagnostics,
                );

                check_setter_path(
                    sig.name.raw(),
                    msg.name.raw(),
                    layout,
                    factor,
                    offset,
                    raw_domain,
                    raw_type_domain,
                    phys_type_domain,
                    diagnostics,
                );
            }
        }
    }
}

fn check_getter_path(
    sig_name: &str,
    msg_name: &str,
    layout: &SignalLayout,
    factor: i128,
    offset: i128,
    raw_domain: (i128, i128),
    phys_type_domain: (i128, i128),
    diagnostics: &mut Diagnostics,
) {
    let Some(product_range) = mul_range(raw_domain, factor) else {
        diagnostics.warning(format!(
            "Signal '{}' in message '{}' can overflow while decoding: raw range [{:?}|{:?}] * factor {} exceeds i128",
            sig_name, msg_name, raw_domain.0, raw_domain.1, factor,
        ));
        return;
    };

    if !range_contains(phys_type_domain, product_range) {
        diagnostics.warning(format!(
            "Signal '{}' in message '{}' can overflow while decoding: intermediate raw * factor range [{}|{}] does not fit inferred physical type range [{}|{}] (size={} bits, factor={}, offset={})",
            sig_name,
            msg_name,
            product_range.0,
            product_range.1,
            phys_type_domain.0,
            phys_type_domain.1,
            layout.size,
            layout.factor,
            layout.offset,
        ));
    }

    let Some(final_range) = add_range(product_range, offset) else {
        diagnostics.warning(format!(
            "Signal '{}' in message '{}' can overflow while decoding: raw * factor + offset exceeds i128",
            sig_name, msg_name,
        ));
        return;
    };

    if !range_contains(phys_type_domain, final_range) {
        diagnostics.warning(format!(
            "Signal '{}' in message '{}' can overflow while decoding: final physical range [{}|{}] does not fit inferred physical type range [{}|{}] (size={} bits, factor={}, offset={})",
            sig_name,
            msg_name,
            final_range.0,
            final_range.1,
            phys_type_domain.0,
            phys_type_domain.1,
            layout.size,
            layout.factor,
            layout.offset,
        ));
    }
}

fn check_setter_path(
    sig_name: &str,
    msg_name: &str,
    layout: &SignalLayout,
    factor: i128,
    offset: i128,
    raw_domain: (i128, i128),
    raw_type_domain: (i128, i128),
    phys_type_domain: (i128, i128),
    diagnostics: &mut Diagnostics,
) {
    // [0|0] commonly means "range unspecified". There is no declared setter
    // domain to prove here, so only the getter path is checked for such signals.
    if layout.min == 0.0 && layout.max == 0.0 {
        return;
    }

    if !is_integer(layout.min) || !is_integer(layout.max) {
        // Fractional declared bounds belong to float physical signals.
        return;
    }

    let declared_range = (layout.min as i128, layout.max as i128);

    if !range_contains(phys_type_domain, declared_range) {
        diagnostics.warning(format!(
            "Signal '{}' in message '{}' declares physical range [{}|{}], which does not fit inferred physical type range [{}|{}]",
            sig_name,
            msg_name,
            declared_range.0,
            declared_range.1,
            phys_type_domain.0,
            phys_type_domain.1,
        ));
        return;
    }

    let Some(pre_div_range) = add_range(declared_range, -offset) else {
        diagnostics.warning(format!(
            "Signal '{}' in message '{}' can overflow while encoding: physical - offset exceeds i128",
            sig_name, msg_name,
        ));
        return;
    };

    if !range_contains(phys_type_domain, pre_div_range) {
        diagnostics.warning(format!(
            "Signal '{}' in message '{}' can overflow while encoding: intermediate physical - offset range [{}|{}] does not fit inferred physical type range [{}|{}] (declared range [{}|{}], offset={})",
            sig_name,
            msg_name,
            pre_div_range.0,
            pre_div_range.1,
            phys_type_domain.0,
            phys_type_domain.1,
            declared_range.0,
            declared_range.1,
            layout.offset,
        ));
    }

    let raw_needed = div_range_trunc_towards_zero(pre_div_range, factor);

    if !range_contains(raw_type_domain, raw_needed) {
        diagnostics.warning(format!(
            "Signal '{}' in message '{}' can overflow while encoding: declared physical range [{}|{}] maps to raw range [{}|{}], which does not fit inferred raw type range [{}|{}]",
            sig_name,
            msg_name,
            declared_range.0,
            declared_range.1,
            raw_needed.0,
            raw_needed.1,
            raw_type_domain.0,
            raw_type_domain.1,
        ));
    }

    if !range_contains(raw_domain, raw_needed) {
        diagnostics.warning(format!(
            "Signal '{}' in message '{}' declares unrepresentable physical values: declared range [{}|{}] maps to raw range [{}|{}], but the actual {}-bit {} raw domain is [{}|{}]",
            sig_name,
            msg_name,
            declared_range.0,
            declared_range.1,
            raw_needed.0,
            raw_needed.1,
            layout.size,
            value_type_name(layout.value_type),
            raw_domain.0,
            raw_domain.1,
        ));
    }
}

fn integer_physical_repr(ty: PhysicalType) -> Option<IntReprType> {
    match ty {
        PhysicalType::Integer(repr) => Some(repr),
        PhysicalType::Enum { repr, .. } => Some(repr),
        PhysicalType::Bool | PhysicalType::Float32 | PhysicalType::Float64 => None,
    }
}

fn actual_raw_domain(layout: &SignalLayout) -> (i128, i128) {
    match layout.value_type {
        ValueType::Unsigned => {
            let max = if layout.size >= 127 {
                i128::MAX
            } else {
                (1i128 << layout.size) - 1
            };
            (0, max)
        }
        ValueType::Signed => {
            if layout.size == 0 {
                (0, 0)
            } else if layout.size >= 128 {
                (i128::MIN, i128::MAX)
            } else {
                let magnitude = 1i128 << (layout.size - 1);
                (-magnitude, magnitude - 1)
            }
        }
    }
}

fn int_repr_domain(repr: IntReprType) -> (i128, i128) {
    match repr {
        IntReprType::U8 => (u8::MIN as i128, u8::MAX as i128),
        IntReprType::U16 => (u16::MIN as i128, u16::MAX as i128),
        IntReprType::U32 => (u32::MIN as i128, u32::MAX as i128),
        IntReprType::U64 => (u64::MIN as i128, u64::MAX as i128),
        IntReprType::I8 => (i8::MIN as i128, i8::MAX as i128),
        IntReprType::I16 => (i16::MIN as i128, i16::MAX as i128),
        IntReprType::I32 => (i32::MIN as i128, i32::MAX as i128),
        IntReprType::I64 => (i64::MIN as i128, i64::MAX as i128),
        IntReprType::I128 => (i128::MIN, i128::MAX),
        IntReprType::U128 => (u128::MIN as i128, u128::MAX as i128),
    }
}

fn range_contains(outer: (i128, i128), inner: (i128, i128)) -> bool {
    outer.0 <= inner.0 && inner.1 <= outer.1
}

fn mul_range(range: (i128, i128), scalar: i128) -> Option<(i128, i128)> {
    let a = range.0.checked_mul(scalar)?;
    let b = range.1.checked_mul(scalar)?;
    Some((a.min(b), a.max(b)))
}

fn add_range(range: (i128, i128), scalar: i128) -> Option<(i128, i128)> {
    let a = range.0.checked_add(scalar)?;
    let b = range.1.checked_add(scalar)?;
    Some((a.min(b), a.max(b)))
}

fn div_range_trunc_towards_zero(range: (i128, i128), divisor: i128) -> (i128, i128) {
    debug_assert_ne!(divisor, 0);
    let a = range.0 / divisor;
    let b = range.1 / divisor;
    (a.min(b), a.max(b))
}

fn is_integer(value: f64) -> bool {
    value.is_finite() && value.fract() == 0.0
}

fn value_type_name(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::Signed => "signed",
        ValueType::Unsigned => "unsigned",
    }
}
