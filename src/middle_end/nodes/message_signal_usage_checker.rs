use crate::middle_end::nodes::{CheckNode, Diagnostics};

pub struct CheckMessageSignalUsage;

impl CheckNode for CheckMessageSignalUsage {
    fn check(&self, file: &crate::DbcFile, diagnostics: &mut Diagnostics) {
        for msg in &file.messages {
            let Some(usage) = &msg.signal_usage else {
                diagnostics.error(format!(
                    "Message '{}' does not have pre-computed signal usage information. Internal error!",
                    msg.name.raw(),
                ));
                continue;
            };

            for case in &usage.cases {
                let ctx = case.context.as_deref();

                for overlap in &case.overlaps {
                    let a = &overlap.first;
                    let b = &overlap.second;

                    match ctx {
                        Some(ctx) => diagnostics.error(format!(
                            "Overlapping signals in message '{}' ({}) : '{}' [{}..{}) overlaps '{}' [{}..{})",
                            msg.name.raw(), ctx, a.signal_name, a.start, a.end, b.signal_name, b.start, b.end
                        )),
                        None => diagnostics.error(format!(
                            "Overlapping signals in message '{}' : '{}' [{}..{}) overlaps '{}' [{}..{})",
                            msg.name.raw(), a.signal_name, a.start, a.end, b.signal_name, b.start, b.end
                        )),
                    }
                }

                check_sum_of_sizes(
                    msg.name.raw(),
                    ctx,
                    msg.size,
                    case.used_bits_sum,
                    diagnostics,
                );
                warn_unused_bits(msg.name.raw(), ctx, &case.unused_bits, diagnostics);
            }
        }
    }
}

fn check_sum_of_sizes(
    msg_name: &str,
    ctx: Option<&str>,
    msg_size_bytes: u64,
    used_bits_sum: u64,
    diagnostics: &mut Diagnostics,
) {
    let msg_bits = msg_size_bytes * 8;

    if used_bits_sum > msg_bits {
        match ctx {
            Some(ctx) => diagnostics.error(format!(
                "Signals in message '{}' ({}) use {} bits in total, which exceeds message size of {} bits",
                msg_name, ctx, used_bits_sum, msg_bits
            )),
            None => diagnostics.error(format!(
                "Signals in message '{}' use {} bits in total, which exceeds message size of {} bits",
                msg_name, used_bits_sum, msg_bits
            )),
        }
    }
}

fn warn_unused_bits(
    msg_name: &str,
    ctx: Option<&str>,
    unused_bits: &[crate::ir::message::MessageSignalBitRange],
    diagnostics: &mut Diagnostics,
) {
    let unused_count: usize = unused_bits.iter().map(|gap| gap.end - gap.start).sum();

    if unused_count == 0 {
        return;
    }

    let gaps_str = unused_bits
        .iter()
        .map(|gap| format!("[{}..{})", gap.start, gap.end))
        .collect::<Vec<_>>()
        .join(", ");

    match ctx {
        Some(ctx) => diagnostics.warning(format!(
            "Message '{}' ({}) has {} unused bit(s): {}",
            msg_name, ctx, unused_count, gaps_str
        )),
        None => diagnostics.warning(format!(
            "Message '{}' has {} unused bit(s): {}",
            msg_name, unused_count, gaps_str
        )),
    }
}
