use crate::{
    ir::{
        message::{
            MessageSignalBitRange, MessageSignalBitSpan, MessageSignalClassification,
            MessageSignalOverlap, MessageSignalUsage, MessageSignalUsageCase,
        },
        signal::SignalIdx,
    },
    middle_end::nodes::TransformationNode,
};

/// Pre-compute per-message bit usage information.
///
/// For a plain message there is a single usage case. For a multiplexed message,
/// each mux value gets one usage case containing the always-active plain signals,
/// the multiplexor signal, and the signals belonging to that mux value.
pub struct AttachMessageSignalUsage;

impl TransformationNode for AttachMessageSignalUsage {
    fn transform(&self, file: &mut crate::DbcFile) {
        for msg_idx in 0..file.messages.len() {
            let msg_size = file.messages[msg_idx].size;
            let usage = match file.messages[msg_idx].classify_signals(&file.signals) {
                MessageSignalClassification::Plain { signals } => MessageSignalUsage {
                    cases: vec![compute_case(file, msg_size, None, &signals)],
                },

                MessageSignalClassification::Multiplexed {
                    plain,
                    mux_signal,
                    muxed,
                } => {
                    let mut base_signal_idxs = plain;
                    base_signal_idxs.push(mux_signal);

                    let cases = muxed
                        .into_iter()
                        .map(|(mux_val, group)| {
                            let mut active = base_signal_idxs.clone();
                            active.extend(group);

                            compute_case(file, msg_size, Some(format!("m{mux_val}")), &active)
                        })
                        .collect();

                    MessageSignalUsage { cases }
                }
            };

            file.messages[msg_idx].signal_usage = Some(usage);
        }
    }
}

fn compute_case(
    file: &crate::DbcFile,
    msg_size_bytes: u64,
    context: Option<String>,
    signal_idxs: &[SignalIdx],
) -> MessageSignalUsageCase {
    let spans = collect_spans(file, signal_idxs);
    let used_bits_sum = spans.iter().map(|s| s.size).sum();
    let overlaps = collect_overlaps(&spans);
    let unused_bits = collect_unused_bits(msg_size_bytes, &spans);

    MessageSignalUsageCase {
        context,
        spans,
        used_bits_sum,
        overlaps,
        unused_bits,
    }
}

fn collect_spans(file: &crate::DbcFile, signal_idxs: &[SignalIdx]) -> Vec<MessageSignalBitSpan> {
    let mut spans = Vec::new();

    for sig_idx in signal_idxs {
        let sig = &file.signals[sig_idx.0];
        let layout = &file.signal_layouts[sig.layout.0];

        spans.push(MessageSignalBitSpan {
            signal_idx: *sig_idx,
            signal_name: sig.name.raw().to_string(),
            start: layout.bitvec_start,
            end: layout.bitvec_end,
            size: layout.size,
        });
    }

    spans.sort_by_key(|s| (s.start, s.end));
    spans
}

fn collect_overlaps(spans: &[MessageSignalBitSpan]) -> Vec<MessageSignalOverlap> {
    let mut overlaps = Vec::new();

    for pair in spans.windows(2) {
        let a = &pair[0];
        let b = &pair[1];

        if b.start < a.end {
            overlaps.push(MessageSignalOverlap {
                first: a.clone(),
                second: b.clone(),
            });
        }
    }

    overlaps
}

fn collect_unused_bits(
    msg_size_bytes: u64,
    spans: &[MessageSignalBitSpan],
) -> Vec<MessageSignalBitRange> {
    let msg_bits = msg_size_bytes as usize * 8;

    if msg_bits == 0 {
        return Vec::new();
    }

    let mut used = vec![false; msg_bits];

    for span in spans {
        let end = span.end.min(msg_bits);
        for bit in span.start.min(msg_bits)..end {
            used[bit] = true;
        }
    }

    collect_gaps(&used)
}

fn collect_gaps(used: &[bool]) -> Vec<MessageSignalBitRange> {
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

        gaps.push(MessageSignalBitRange { start, end: i });
    }

    gaps
}
