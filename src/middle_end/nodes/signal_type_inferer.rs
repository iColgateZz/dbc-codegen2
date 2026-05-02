use std::cmp::{max, min};

use crate::{
    ir::{
        signal::Signal,
        signal_extended_value_type::ExtendedValueType,
        signal_layout::{SignalLayout, ValueType},
        signal_value_enum::SignalValueEnum,
        signal_value_type::{EnumCoverage, IntReprType, PhysicalType, RawType},
    },
    middle_end::nodes::TransformationNode,
};

/// Infer signal raw and physical types.
///
/// The inferred types intentionally separate three concepts:
///
/// - `raw_type`: the DBC raw numeric interpretation. This follows `SIG_VALTYPE_`
///   for floats and otherwise follows the signal size and signedness.
/// - `physical_type`: the public Rust type used by generated getters and setters.
///   For integer signals this is inferred from the full scaled raw range, not from
///   the DBC min/max metadata. DBC files commonly use `[0|0]` to mean
///   "unspecified range", so min/max are not reliable enough for type selection.
/// - storage type: Rust codegen derives this from `raw_type` as the unsigned type
///   with the same width. It is not stored in the IR.
pub struct InferSignalTypes;

impl TransformationNode for InferSignalTypes {
    fn transform(&self, file: &mut crate::DbcFile) {
        for sig in &mut file.signals {
            let sig_layout = &file.signal_layouts[sig.layout.0];
            let sve_option = sig
                .signal_value_enum_idx
                .map(|idx| &file.signal_value_enums[idx.0]);
            sig.raw_type = infer_raw_type(sig, sig_layout);
            sig.physical_type = infer_physical_type(sig, sig_layout, sve_option);
        }
    }
}

fn infer_raw_type(sig: &Signal, sig_layout: &SignalLayout) -> RawType {
    match sig.extended_type {
        ExtendedValueType::Float32 => RawType::Float32,
        ExtendedValueType::Double64 => RawType::Float64,
        ExtendedValueType::Integer => {
            let signed = matches!(sig_layout.value_type, ValueType::Signed);
            RawType::Integer(IntReprType::from_size_sign(sig_layout.size, signed))
        }
    }
}

fn infer_physical_type(
    sig: &Signal,
    sig_layout: &SignalLayout,
    sve_option: Option<&SignalValueEnum>,
) -> PhysicalType {
    if let Some(sve) = sve_option {
        let coverage = enum_coverage(sig_layout.size, sve.variants.len());
        let signed = matches!(sig_layout.value_type, ValueType::Signed);
        let repr = IntReprType::from_size_sign(sig_layout.size, signed);
        return PhysicalType::Enum { coverage, repr };
    }

    match sig.raw_type {
        RawType::Float32 => PhysicalType::Float32,
        RawType::Float64 => PhysicalType::Float64,
        RawType::Integer(_) => {
            if is_bool_signal(sig_layout) {
                PhysicalType::Bool
            } else if is_float_scaled(sig_layout) {
                // The rest of the Rust code generator currently uses f32 for
                // integer-backed scaled physical values. Raw SIG_VALTYPE_ floats
                // are handled above and keep their explicit precision.
                PhysicalType::Float32
            } else {
                let (low, high) = scaled_integer_range(sig_layout)
                    .expect("integer signal range should fit into i128");
                PhysicalType::Integer(IntReprType::from_min_max(low, high))
            }
        }
    }
}

fn enum_coverage(size: u64, variant_count: usize) -> EnumCoverage {
    match 1u128.checked_shl(size as u32) {
        Some(possible_values) if variant_count as u128 == possible_values => {
            EnumCoverage::Exhaustive
        }
        _ => EnumCoverage::Partial,
    }
}

fn is_bool_signal(sig_layout: &SignalLayout) -> bool {
    sig_layout.size == 1 && sig_layout.factor == 1.0 && sig_layout.offset == 0.0
}

fn is_float_scaled(sig_layout: &SignalLayout) -> bool {
    sig_layout.factor.fract() != 0.0 || sig_layout.offset.fract() != 0.0
}

fn scaled_integer_range(sig_layout: &SignalLayout) -> Option<(i128, i128)> {
    let (raw_low, raw_high) = raw_integer_range(sig_layout)?;
    let factor = sig_layout.factor as i128;
    let offset = sig_layout.offset as i128;

    let a = raw_low.checked_mul(factor)?.checked_add(offset)?;
    let b = raw_high.checked_mul(factor)?.checked_add(offset)?;

    Some((min(a, b), max(a, b)))
}

fn raw_integer_range(sig_layout: &SignalLayout) -> Option<(i128, i128)> {
    let size = sig_layout.size;

    if size == 0 || size > 128 {
        return None;
    }

    if matches!(sig_layout.value_type, ValueType::Signed) {
        let high_bit = size.checked_sub(1)?;
        let magnitude = 1i128.checked_shl(high_bit as u32)?;
        Some((magnitude.checked_neg()?, magnitude.checked_sub(1)?))
    } else {
        let values = 1i128.checked_shl(size as u32)?;
        Some((0, values.checked_sub(1)?))
    }
}
