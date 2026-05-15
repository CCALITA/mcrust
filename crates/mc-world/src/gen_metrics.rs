//! Chunk generation performance metrics.

/// Tracks chunk generation performance statistics.
#[derive(Debug, Clone)]
pub struct GenMetrics {
    pub chunks_generated: u64,
    pub total_gen_time_ms: f64,
    pub avg_gen_time_ms: f64,
    pub peak_gen_time_ms: f64,
}

impl GenMetrics {
    /// Creates a new `GenMetrics` with all values zeroed.
    pub fn new() -> Self {
        Self {
            chunks_generated: 0,
            total_gen_time_ms: 0.0,
            avg_gen_time_ms: 0.0,
            peak_gen_time_ms: 0.0,
        }
    }
}

impl Default for GenMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Records a chunk generation with the given duration in milliseconds.
pub fn record_generation(metrics: &GenMetrics, duration_ms: f64) -> GenMetrics {
    let chunks_generated = metrics.chunks_generated + 1;
    let total_gen_time_ms = metrics.total_gen_time_ms + duration_ms;
    let avg_gen_time_ms = total_gen_time_ms / chunks_generated as f64;
    let peak_gen_time_ms = if duration_ms > metrics.peak_gen_time_ms {
        duration_ms
    } else {
        metrics.peak_gen_time_ms
    };

    GenMetrics {
        chunks_generated,
        total_gen_time_ms,
        avg_gen_time_ms,
        peak_gen_time_ms,
    }
}

/// Resets the peak generation time to zero.
pub fn reset_peak(metrics: &GenMetrics) -> GenMetrics {
    GenMetrics {
        peak_gen_time_ms: 0.0,
        ..metrics.clone()
    }
}

/// Returns the chunk generation rate per second.
///
/// Returns 0.0 if `elapsed_s` is zero or negative.
pub fn gen_rate_per_second(metrics: &GenMetrics, elapsed_s: f64) -> f64 {
    if elapsed_s <= 0.0 {
        return 0.0;
    }
    metrics.chunks_generated as f64 / elapsed_s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_metrics_are_zeroed() {
        let m = GenMetrics::new();
        assert_eq!(m.chunks_generated, 0);
        assert_eq!(m.total_gen_time_ms, 0.0);
        assert_eq!(m.avg_gen_time_ms, 0.0);
        assert_eq!(m.peak_gen_time_ms, 0.0);
    }

    #[test]
    fn record_single_generation() {
        let m = GenMetrics::new();
        let m = record_generation(&m, 12.5);
        assert_eq!(m.chunks_generated, 1);
        assert_eq!(m.total_gen_time_ms, 12.5);
        assert_eq!(m.avg_gen_time_ms, 12.5);
        assert_eq!(m.peak_gen_time_ms, 12.5);
    }

    #[test]
    fn record_multiple_generations() {
        let m = GenMetrics::new();
        let m = record_generation(&m, 10.0);
        let m = record_generation(&m, 20.0);
        let m = record_generation(&m, 15.0);
        assert_eq!(m.chunks_generated, 3);
        assert_eq!(m.total_gen_time_ms, 45.0);
        assert!((m.avg_gen_time_ms - 15.0).abs() < f64::EPSILON);
        assert_eq!(m.peak_gen_time_ms, 20.0);
    }

    #[test]
    fn peak_tracks_maximum() {
        let m = GenMetrics::new();
        let m = record_generation(&m, 30.0);
        let m = record_generation(&m, 10.0);
        assert_eq!(m.peak_gen_time_ms, 30.0);
    }

    #[test]
    fn reset_peak_clears_peak_only() {
        let m = GenMetrics::new();
        let m = record_generation(&m, 25.0);
        let m = reset_peak(&m);
        assert_eq!(m.peak_gen_time_ms, 0.0);
        assert_eq!(m.chunks_generated, 1);
        assert_eq!(m.total_gen_time_ms, 25.0);
    }

    #[test]
    fn gen_rate_per_second_calculates_correctly() {
        let m = GenMetrics::new();
        let m = record_generation(&m, 5.0);
        let m = record_generation(&m, 5.0);
        let m = record_generation(&m, 5.0);
        let rate = gen_rate_per_second(&m, 1.5);
        assert!((rate - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gen_rate_zero_elapsed() {
        let m = GenMetrics::new();
        let m = record_generation(&m, 5.0);
        assert_eq!(gen_rate_per_second(&m, 0.0), 0.0);
    }

    #[test]
    fn gen_rate_negative_elapsed() {
        let m = GenMetrics::new();
        let m = record_generation(&m, 5.0);
        assert_eq!(gen_rate_per_second(&m, -1.0), 0.0);
    }

    #[test]
    fn default_is_same_as_new() {
        let m = GenMetrics::default();
        assert_eq!(m.chunks_generated, 0);
        assert_eq!(m.peak_gen_time_ms, 0.0);
    }
}
