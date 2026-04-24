//! Network latency meter: track ping samples, jitter, and connection quality.

/// Rolling window of network latency samples.
#[derive(Debug, Clone)]
pub struct LatencyMeter {
    pub samples: Vec<u32>,
    pub max_samples: usize,
    pub last_ping_ms: u32,
}

impl Default for LatencyMeter {
    fn default() -> Self {
        Self::new()
    }
}

impl LatencyMeter {
    /// Create a new meter with a 20-sample rolling window.
    pub fn new() -> Self {
        Self {
            samples: Vec::with_capacity(20),
            max_samples: 20,
            last_ping_ms: 0,
        }
    }

    /// Record a new ping sample, dropping the oldest if the window is full.
    pub fn record_ping(&mut self, latency_ms: u32) {
        self.last_ping_ms = latency_ms;
        self.samples.push(latency_ms);
        while self.samples.len() > self.max_samples {
            self.samples.remove(0);
        }
    }

    /// Average latency across recorded samples (0 if empty).
    pub fn average(&self) -> u32 {
        if self.samples.is_empty() {
            return 0;
        }
        let sum: u64 = self.samples.iter().map(|&s| s as u64).sum();
        (sum / self.samples.len() as u64) as u32
    }

    /// Jitter as the population standard deviation of samples.
    pub fn jitter(&self) -> u32 {
        if self.samples.is_empty() {
            return 0;
        }
        let avg = self.average() as f64;
        let n = self.samples.len() as f64;
        let variance: f64 = self
            .samples
            .iter()
            .map(|&s| {
                let d = s as f64 - avg;
                d * d
            })
            .sum::<f64>()
            / n;
        variance.sqrt() as u32
    }
}

/// Qualitative bucket for a given ping latency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

/// Map a latency (ms) to a [`ConnectionQuality`] bucket.
pub fn connection_quality(latency_ms: u32) -> ConnectionQuality {
    if latency_ms < 50 {
        ConnectionQuality::Excellent
    } else if latency_ms < 100 {
        ConnectionQuality::Good
    } else if latency_ms < 200 {
        ConnectionQuality::Fair
    } else if latency_ms < 500 {
        ConnectionQuality::Poor
    } else {
        ConnectionQuality::Critical
    }
}

/// RGB color for a given connection quality (linear 0..=1 per channel).
pub fn quality_color(q: ConnectionQuality) -> [f32; 3] {
    match q {
        ConnectionQuality::Excellent => [0.0, 1.0, 0.0],   // green
        ConnectionQuality::Good => [0.6, 1.0, 0.0],        // yellow-green
        ConnectionQuality::Fair => [1.0, 1.0, 0.0],        // yellow
        ConnectionQuality::Poor => [1.0, 0.5, 0.0],        // orange
        ConnectionQuality::Critical => [1.0, 0.0, 0.0],    // red
    }
}

/// Signal bar count (0-5) for a given connection quality.
pub fn quality_bars(q: ConnectionQuality) -> u8 {
    match q {
        ConnectionQuality::Excellent => 5,
        ConnectionQuality::Good => 4,
        ConnectionQuality::Fair => 3,
        ConnectionQuality::Poor => 2,
        ConnectionQuality::Critical => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_meter_has_empty_samples_and_cap_20() {
        let m = LatencyMeter::new();
        assert!(m.samples.is_empty());
        assert_eq!(m.max_samples, 20);
        assert_eq!(m.last_ping_ms, 0);
    }

    #[test]
    fn record_ping_updates_last_and_pushes_sample() {
        let mut m = LatencyMeter::new();
        m.record_ping(42);
        assert_eq!(m.last_ping_ms, 42);
        assert_eq!(m.samples, vec![42]);
    }

    #[test]
    fn record_ping_drops_oldest_over_cap() {
        let mut m = LatencyMeter::new();
        for i in 0..25u32 {
            m.record_ping(i);
        }
        assert_eq!(m.samples.len(), 20);
        assert_eq!(m.samples[0], 5);
        assert_eq!(m.samples[19], 24);
        assert_eq!(m.last_ping_ms, 24);
    }

    #[test]
    fn average_of_empty_is_zero() {
        let m = LatencyMeter::new();
        assert_eq!(m.average(), 0);
    }

    #[test]
    fn average_computes_mean() {
        let mut m = LatencyMeter::new();
        for v in [10, 20, 30, 40] {
            m.record_ping(v);
        }
        assert_eq!(m.average(), 25);
    }

    #[test]
    fn jitter_of_empty_is_zero() {
        let m = LatencyMeter::new();
        assert_eq!(m.jitter(), 0);
    }

    #[test]
    fn jitter_of_constant_samples_is_zero() {
        let mut m = LatencyMeter::new();
        for _ in 0..5 {
            m.record_ping(100);
        }
        assert_eq!(m.jitter(), 0);
    }

    #[test]
    fn jitter_nonzero_for_varied_samples() {
        let mut m = LatencyMeter::new();
        for v in [10, 20, 30, 40, 50] {
            m.record_ping(v);
        }
        // population stddev of [10,20,30,40,50] is ~14.14
        assert_eq!(m.jitter(), 14);
    }

    #[test]
    fn connection_quality_buckets() {
        assert_eq!(connection_quality(0), ConnectionQuality::Excellent);
        assert_eq!(connection_quality(49), ConnectionQuality::Excellent);
        assert_eq!(connection_quality(50), ConnectionQuality::Good);
        assert_eq!(connection_quality(99), ConnectionQuality::Good);
        assert_eq!(connection_quality(100), ConnectionQuality::Fair);
        assert_eq!(connection_quality(199), ConnectionQuality::Fair);
        assert_eq!(connection_quality(200), ConnectionQuality::Poor);
        assert_eq!(connection_quality(499), ConnectionQuality::Poor);
        assert_eq!(connection_quality(500), ConnectionQuality::Critical);
        assert_eq!(connection_quality(10_000), ConnectionQuality::Critical);
    }

    #[test]
    fn quality_color_matches_spec() {
        assert_eq!(quality_color(ConnectionQuality::Excellent), [0.0, 1.0, 0.0]);
        assert_eq!(quality_color(ConnectionQuality::Good), [0.6, 1.0, 0.0]);
        assert_eq!(quality_color(ConnectionQuality::Fair), [1.0, 1.0, 0.0]);
        assert_eq!(quality_color(ConnectionQuality::Poor), [1.0, 0.5, 0.0]);
        assert_eq!(quality_color(ConnectionQuality::Critical), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn quality_bars_monotonic() {
        assert_eq!(quality_bars(ConnectionQuality::Excellent), 5);
        assert_eq!(quality_bars(ConnectionQuality::Good), 4);
        assert_eq!(quality_bars(ConnectionQuality::Fair), 3);
        assert_eq!(quality_bars(ConnectionQuality::Poor), 2);
        assert_eq!(quality_bars(ConnectionQuality::Critical), 1);
    }
}
