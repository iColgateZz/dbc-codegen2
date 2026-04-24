use crate::{
    ir::{
        message::MessageSignalClassification,
        signal::SignalIdx,
    },
    middle_end::nodes::{CheckNode, Diagnostics},
};

pub struct CheckMessageSignalUsage;

#[derive(Clone)]
struct SpanInfo<'a> {
    sig_name: &'a str,
    start: usize,
    end: usize,
    size: u64,
}

//TODO: add another node to ensure there is only 1 multiplexor per message
//      give errors on MultiplexorAndMultiplexed signals?
impl CheckNode for CheckMessageSignalUsage {
    fn check(&self, file: &crate::DbcFile, diagnostics: &mut Diagnostics) {
        for msg in &file.messages {
            match msg.classify_signals(&file.signals) {
                MessageSignalClassification::Plain { signals } => {
                    let spans = collect_spans(file, &signals);
                    check_overlaps(msg.name.raw(), None, &spans, diagnostics);
                    check_sum_of_sizes(msg.name.raw(), None, msg.size, &spans, diagnostics);
                    warn_unused_bits(msg.name.raw(), None, msg.size, &spans, diagnostics);
                }

                MessageSignalClassification::Multiplexed {
                    plain,
                    mux_signal,
                    muxed,
                } => {
                    let mut base_signal_idxs = plain;
                    base_signal_idxs.push(mux_signal);

                    for (mux_val, group) in muxed {
                        let mut active = base_signal_idxs.clone();
                        active.extend(group);

                        let spans = collect_spans(file, &active);
                        let ctx = format!("m{mux_val}");

                        check_overlaps(msg.name.raw(), Some(&ctx), &spans, diagnostics);
                        check_sum_of_sizes(msg.name.raw(), Some(&ctx), msg.size, &spans, diagnostics);
                        warn_unused_bits(msg.name.raw(), Some(&ctx), msg.size, &spans, diagnostics);
                    }
                }
            }
        }
    }
}

fn collect_spans<'a>(file: &'a crate::DbcFile, signal_idxs: &[SignalIdx]) -> Vec<SpanInfo<'a>> {
    let mut spans = Vec::new();

    for sig_idx in signal_idxs {
        let sig = &file.signals[sig_idx.0];
        let layout = &file.signal_layouts[sig.layout.0];

        spans.push(SpanInfo {
            sig_name: sig.name.raw(),
            start: layout.bitvec_start,
            end: layout.bitvec_end,
            size: layout.size,
        });
    }

    spans.sort_by_key(|s| (s.start, s.end));
    spans
}

fn check_overlaps(
    msg_name: &str,
    ctx: Option<&str>,
    spans: &[SpanInfo<'_>],
    diagnostics: &mut Diagnostics,
) {
    for pair in spans.windows(2) {
        let a = &pair[0];
        let b = &pair[1];

        if b.start < a.end {
            match ctx {
                Some(ctx) => diagnostics.error(format!(
                    "Overlapping signals in message '{}' ({}) : '{}' [{}..{}) overlaps '{}' [{}..{})",
                    msg_name, ctx, a.sig_name, a.start, a.end, b.sig_name, b.start, b.end
                )),
                None => diagnostics.error(format!(
                    "Overlapping signals in message '{}' : '{}' [{}..{}) overlaps '{}' [{}..{})",
                    msg_name, a.sig_name, a.start, a.end, b.sig_name, b.start, b.end
                )),
            }
        }
    }
}

fn check_sum_of_sizes(
    msg_name: &str,
    ctx: Option<&str>,
    msg_size_bytes: u64,
    spans: &[SpanInfo<'_>],
    diagnostics: &mut Diagnostics,
) {
    let sum_bits: u64 = spans.iter().map(|s| s.size).sum();
    let msg_bits = msg_size_bytes * 8;

    if sum_bits > msg_bits {
        match ctx {
            Some(ctx) => diagnostics.error(format!(
                "Signals in message '{}' ({}) use {} bits in total, which exceeds message size of {} bits",
                msg_name, ctx, sum_bits, msg_bits
            )),
            None => diagnostics.error(format!(
                "Signals in message '{}' use {} bits in total, which exceeds message size of {} bits",
                msg_name, sum_bits, msg_bits
            )),
        }
    }
}

fn warn_unused_bits(
    msg_name: &str,
    ctx: Option<&str>,
    msg_size_bytes: u64,
    spans: &[SpanInfo<'_>],
    diagnostics: &mut Diagnostics,
) {
    let msg_bits = msg_size_bytes as usize * 8;

    if msg_bits == 0 {
        return;
    }

    let mut used = vec![false; msg_bits];

    for span in spans {
        let end = span.end.min(msg_bits);
        for bit in span.start.min(msg_bits)..end {
            used[bit] = true;
        }
    }

    let unused_count = used.iter().filter(|v| !**v).count();

    if unused_count == 0 {
        return;
    }

    let gaps = collect_gaps(&used);
    let gaps_str = gaps
        .into_iter()
        .map(|(start, end)| format!("[{}..{})", start, end))
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

fn collect_gaps(used: &[bool]) -> Vec<(usize, usize)> {
    let mut gaps = Vec::new();
    let mut i = 0;

    while i < used.len() {
        if used[i] {
            i += 1;
            continue;
        }

        let start = i;
        while i < used.len() && !used[i] {
            i += 1;
        }

        gaps.push((start, i));
    }

    gaps
}