//! Frame time profiler for render performance analysis.

/// A named section of a frame with its measured duration.
#[derive(Debug, Clone)]
pub struct ProfileSection {
    pub name: String,
    pub duration_ms: f32,
}

/// Tracks per-frame timing data and provides statistical analysis.
#[derive(Debug)]
pub struct FrameProfiler {
    sections: Vec<ProfileSection>,
    frame_times: Vec<f32>,
    max_history: usize,
}

impl FrameProfiler {
    /// Create a new profiler that retains up to `max_history` frame times.
    pub fn new(max_history: usize) -> Self {
        Self {
            sections: Vec::new(),
            frame_times: Vec::with_capacity(max_history),
            max_history,
        }
    }

    /// Clear per-frame section data to start a new frame.
    pub fn begin_frame(&mut self) {
        self.sections.clear();
    }

    /// Record a named section's duration for the current frame.
    pub fn record_section(&mut self, name: &str, duration_ms: f32) {
        self.sections.push(ProfileSection {
            name: name.to_string(),
            duration_ms,
        });
    }

    /// Finalize the frame, storing the total frame time in history.
    pub fn end_frame(&mut self, total_ms: f32) {
        if self.frame_times.len() >= self.max_history {
            self.frame_times.remove(0);
        }
        self.frame_times.push(total_ms);
    }

    /// Average frame time over the last `last_n` frames.
    pub fn average_frame_time(&self, last_n: usize) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let count = last_n.min(self.frame_times.len());
        let start = self.frame_times.len() - count;
        let sum: f32 = self.frame_times[start..].iter().sum();
        sum / count as f32
    }

    /// Return the p-th percentile frame time (p in 0.0..=100.0).
    pub fn percentile_frame_time(&self, p: f32) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let mut sorted = self.frame_times.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let index = ((p / 100.0) * (sorted.len() as f32 - 1.0))
            .clamp(0.0, (sorted.len() - 1) as f32) as usize;
        sorted[index]
    }

    /// Return a report of the current frame's sections as (name, duration_ms) pairs.
    pub fn frame_report(&self) -> Vec<(&str, f32)> {
        self.sections
            .iter()
            .map(|s| (s.name.as_str(), s.duration_ms))
            .collect()
    }

    /// Heuristic: the frame is GPU-bound if the "gpu" section exceeds 50% of total section time.
    pub fn is_gpu_bound(&self) -> bool {
        let total: f32 = self.sections.iter().map(|s| s.duration_ms).sum();
        if total <= 0.0 {
            return false;
        }
        let gpu_time: f32 = self
            .sections
            .iter()
            .filter(|s| s.name.to_lowercase().contains("gpu"))
            .map(|s| s.duration_ms)
            .sum();
        gpu_time / total > 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_profiler_has_empty_history() {
        let profiler = FrameProfiler::new(100);
        assert_eq!(profiler.average_frame_time(10), 0.0);
        assert_eq!(profiler.percentile_frame_time(99.0), 0.0);
        assert!(profiler.frame_report().is_empty());
    }

    #[test]
    fn records_sections_and_reports() {
        let mut profiler = FrameProfiler::new(100);
        profiler.begin_frame();
        profiler.record_section("mesh", 3.0);
        profiler.record_section("lighting", 2.0);
        let report = profiler.frame_report();
        assert_eq!(report.len(), 2);
        assert_eq!(report[0], ("mesh", 3.0));
        assert_eq!(report[1], ("lighting", 2.0));
    }

    #[test]
    fn begin_frame_clears_sections() {
        let mut profiler = FrameProfiler::new(100);
        profiler.begin_frame();
        profiler.record_section("pass1", 1.0);
        profiler.begin_frame();
        assert!(profiler.frame_report().is_empty());
    }

    #[test]
    fn end_frame_stores_history() {
        let mut profiler = FrameProfiler::new(3);
        profiler.end_frame(10.0);
        profiler.end_frame(20.0);
        profiler.end_frame(30.0);
        assert!((profiler.average_frame_time(3) - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn history_respects_max() {
        let mut profiler = FrameProfiler::new(2);
        profiler.end_frame(10.0);
        profiler.end_frame(20.0);
        profiler.end_frame(30.0);
        // oldest (10.0) evicted
        assert!((profiler.average_frame_time(10) - 25.0).abs() < f32::EPSILON);
    }

    #[test]
    fn average_frame_time_with_last_n() {
        let mut profiler = FrameProfiler::new(100);
        profiler.end_frame(10.0);
        profiler.end_frame(20.0);
        profiler.end_frame(30.0);
        assert!((profiler.average_frame_time(2) - 25.0).abs() < f32::EPSILON);
    }

    #[test]
    fn percentile_frame_time_median() {
        let mut profiler = FrameProfiler::new(100);
        for t in [5.0, 10.0, 15.0, 20.0, 25.0] {
            profiler.end_frame(t);
        }
        assert!((profiler.percentile_frame_time(50.0) - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn percentile_frame_time_extremes() {
        let mut profiler = FrameProfiler::new(100);
        profiler.end_frame(5.0);
        profiler.end_frame(10.0);
        assert!((profiler.percentile_frame_time(0.0) - 5.0).abs() < f32::EPSILON);
        assert!((profiler.percentile_frame_time(100.0) - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn is_gpu_bound_true_when_gpu_dominant() {
        let mut profiler = FrameProfiler::new(100);
        profiler.begin_frame();
        profiler.record_section("gpu_render", 8.0);
        profiler.record_section("cpu_sim", 2.0);
        assert!(profiler.is_gpu_bound());
    }

    #[test]
    fn is_gpu_bound_false_when_cpu_dominant() {
        let mut profiler = FrameProfiler::new(100);
        profiler.begin_frame();
        profiler.record_section("gpu_render", 2.0);
        profiler.record_section("cpu_sim", 8.0);
        assert!(!profiler.is_gpu_bound());
    }

    #[test]
    fn is_gpu_bound_false_when_no_sections() {
        let profiler = FrameProfiler::new(100);
        assert!(!profiler.is_gpu_bound());
    }
}
