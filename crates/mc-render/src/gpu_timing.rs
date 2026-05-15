//! GPU timing query helpers for profiling render passes.

/// A single GPU timestamp measurement.
#[derive(Debug, Clone)]
pub struct GpuTimestamp {
    pub label: String,
    pub start_ns: u64,
    pub end_ns: u64,
}

/// A collection of GPU timestamp measurements for a frame.
#[derive(Debug, Clone)]
pub struct GpuTimingReport {
    pub timestamps: Vec<GpuTimestamp>,
}

/// Returns the duration of a single timestamp in milliseconds.
pub fn duration_ms(ts: &GpuTimestamp) -> f32 {
    (ts.end_ns.saturating_sub(ts.start_ns)) as f32 / 1_000_000.0
}

/// Returns the total GPU time across all timestamps in milliseconds.
pub fn total_gpu_ms(report: &GpuTimingReport) -> f32 {
    report.timestamps.iter().map(|ts| duration_ms(ts)).sum()
}

/// Formats a GPU timing report as a human-readable string.
pub fn format_gpu_report(report: &GpuTimingReport) -> String {
    let mut lines = Vec::with_capacity(report.timestamps.len() + 1);
    for ts in &report.timestamps {
        lines.push(format!("  {}: {:.2}ms", ts.label, duration_ms(ts)));
    }
    lines.push(format!("  Total: {:.2}ms", total_gpu_ms(report)));
    lines.join("\n")
}

/// Returns true if GPU time exceeds 60% of frame time, indicating a GPU bottleneck.
pub fn is_gpu_bottleneck(gpu_ms: f32, frame_ms: f32) -> bool {
    if frame_ms <= 0.0 {
        return false;
    }
    (gpu_ms / frame_ms) > 0.6
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_timestamp(label: &str, start_ns: u64, end_ns: u64) -> GpuTimestamp {
        GpuTimestamp {
            label: label.to_string(),
            start_ns,
            end_ns,
        }
    }

    fn sample_report() -> GpuTimingReport {
        GpuTimingReport {
            timestamps: vec![
                sample_timestamp("shadow", 0, 2_000_000),
                sample_timestamp("geometry", 2_000_000, 5_000_000),
                sample_timestamp("lighting", 5_000_000, 6_500_000),
            ],
        }
    }

    #[test]
    fn duration_ms_computes_correctly() {
        let ts = sample_timestamp("test", 1_000_000, 3_500_000);
        assert!((duration_ms(&ts) - 2.5).abs() < 0.001);
    }

    #[test]
    fn duration_ms_handles_zero_duration() {
        let ts = sample_timestamp("zero", 100, 100);
        assert!((duration_ms(&ts)).abs() < 0.001);
    }

    #[test]
    fn duration_ms_saturates_on_underflow() {
        let ts = sample_timestamp("bad", 500, 100);
        assert!((duration_ms(&ts)).abs() < 0.001);
    }

    #[test]
    fn total_gpu_ms_sums_all_timestamps() {
        let report = sample_report();
        // 2.0 + 3.0 + 1.5 = 6.5
        assert!((total_gpu_ms(&report) - 6.5).abs() < 0.001);
    }

    #[test]
    fn total_gpu_ms_empty_report() {
        let report = GpuTimingReport { timestamps: vec![] };
        assert!((total_gpu_ms(&report)).abs() < 0.001);
    }

    #[test]
    fn format_gpu_report_includes_all_labels() {
        let report = sample_report();
        let output = format_gpu_report(&report);
        assert!(output.contains("shadow"));
        assert!(output.contains("geometry"));
        assert!(output.contains("lighting"));
        assert!(output.contains("Total"));
    }

    #[test]
    fn is_gpu_bottleneck_above_threshold() {
        assert!(is_gpu_bottleneck(10.0, 15.0)); // 66.7%
    }

    #[test]
    fn is_gpu_bottleneck_below_threshold() {
        assert!(!is_gpu_bottleneck(5.0, 16.7)); // ~30%
    }

    #[test]
    fn is_gpu_bottleneck_at_boundary() {
        assert!(!is_gpu_bottleneck(6.0, 10.0)); // exactly 60%
    }

    #[test]
    fn is_gpu_bottleneck_zero_frame_time() {
        assert!(!is_gpu_bottleneck(5.0, 0.0));
    }
}
