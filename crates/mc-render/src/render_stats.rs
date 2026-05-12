//! Render statistics tracking for frame performance monitoring.

/// Per-frame rendering statistics.
#[derive(Debug, Clone)]
pub struct RenderStats {
    pub draw_calls: u32,
    pub vertices: u64,
    pub triangles: u64,
    pub chunks_rendered: u32,
    pub chunks_culled: u32,
    pub frame_time_ms: f32,
    pub fps: f32,
}

impl RenderStats {
    /// Creates a new zeroed `RenderStats`.
    pub fn new() -> Self {
        Self {
            draw_calls: 0,
            vertices: 0,
            triangles: 0,
            chunks_rendered: 0,
            chunks_culled: 0,
            frame_time_ms: 0.0,
            fps: 0.0,
        }
    }
}

impl Default for RenderStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns a new `RenderStats` with updated frame time and FPS from `dt` (in seconds).
pub fn update_frame_time(stats: &RenderStats, dt: f32) -> RenderStats {
    let frame_time_ms = dt * 1000.0;
    let fps = if dt > 0.0 { 1.0 / dt } else { 0.0 };
    RenderStats {
        frame_time_ms,
        fps,
        ..stats.clone()
    }
}

/// Calculates average FPS from a slice of frame times (in milliseconds).
pub fn calculate_fps(frame_times: &[f32]) -> f32 {
    if frame_times.is_empty() {
        return 0.0;
    }
    let avg_ms = frame_times.iter().sum::<f32>() / frame_times.len() as f32;
    if avg_ms > 0.0 {
        1000.0 / avg_ms
    } else {
        0.0
    }
}

/// Formats render stats into a human-readable string.
pub fn format_stats(stats: &RenderStats) -> String {
    format!(
        "FPS: {:.1} | Frame: {:.2}ms | Draw calls: {} | Verts: {} | Tris: {} | Chunks: {}/{} (rendered/culled)",
        stats.fps,
        stats.frame_time_ms,
        stats.draw_calls,
        stats.vertices,
        stats.triangles,
        stats.chunks_rendered,
        stats.chunks_culled,
    )
}

/// Returns a warning if stats indicate performance problems.
pub fn stats_warning(stats: &RenderStats) -> Option<&'static str> {
    if stats.fps > 0.0 && stats.fps < 30.0 {
        Some("Low FPS: performance is below 30 FPS")
    } else if stats.draw_calls > 10_000 {
        Some("High draw call count: consider batching geometry")
    } else if stats.frame_time_ms > 33.3 {
        Some("High frame time: frame took longer than 33ms")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_returns_zeroed_stats() {
        let stats = RenderStats::new();
        assert_eq!(stats.draw_calls, 0);
        assert_eq!(stats.vertices, 0);
        assert_eq!(stats.triangles, 0);
        assert_eq!(stats.chunks_rendered, 0);
        assert_eq!(stats.chunks_culled, 0);
        assert_eq!(stats.frame_time_ms, 0.0);
        assert_eq!(stats.fps, 0.0);
    }

    #[test]
    fn update_frame_time_sets_ms_and_fps() {
        let stats = RenderStats::new();
        let updated = update_frame_time(&stats, 0.016);
        assert!((updated.frame_time_ms - 16.0).abs() < 0.01);
        assert!((updated.fps - 62.5).abs() < 0.1);
    }

    #[test]
    fn update_frame_time_zero_dt() {
        let stats = RenderStats::new();
        let updated = update_frame_time(&stats, 0.0);
        assert_eq!(updated.fps, 0.0);
        assert_eq!(updated.frame_time_ms, 0.0);
    }

    #[test]
    fn update_frame_time_preserves_other_fields() {
        let stats = RenderStats {
            draw_calls: 42,
            vertices: 1000,
            triangles: 500,
            chunks_rendered: 10,
            chunks_culled: 5,
            frame_time_ms: 0.0,
            fps: 0.0,
        };
        let updated = update_frame_time(&stats, 0.01);
        assert_eq!(updated.draw_calls, 42);
        assert_eq!(updated.vertices, 1000);
        assert_eq!(updated.triangles, 500);
        assert_eq!(updated.chunks_rendered, 10);
        assert_eq!(updated.chunks_culled, 5);
    }

    #[test]
    fn calculate_fps_from_frame_times() {
        let times = vec![16.0, 16.0, 16.0];
        let fps = calculate_fps(&times);
        assert!((fps - 62.5).abs() < 0.1);
    }

    #[test]
    fn calculate_fps_empty_slice() {
        assert_eq!(calculate_fps(&[]), 0.0);
    }

    #[test]
    fn calculate_fps_zero_times() {
        assert_eq!(calculate_fps(&[0.0, 0.0]), 0.0);
    }

    #[test]
    fn format_stats_contains_key_info() {
        let stats = RenderStats {
            draw_calls: 100,
            vertices: 50000,
            triangles: 25000,
            chunks_rendered: 64,
            chunks_culled: 128,
            frame_time_ms: 16.67,
            fps: 60.0,
        };
        let output = format_stats(&stats);
        assert!(output.contains("60.0"));
        assert!(output.contains("16.67"));
        assert!(output.contains("100"));
        assert!(output.contains("50000"));
        assert!(output.contains("25000"));
        assert!(output.contains("64"));
        assert!(output.contains("128"));
    }

    #[test]
    fn stats_warning_low_fps() {
        let stats = RenderStats { fps: 20.0, ..RenderStats::new() };
        assert!(stats_warning(&stats).unwrap().contains("Low FPS"));
    }

    #[test]
    fn stats_warning_high_draw_calls() {
        let stats = RenderStats {
            draw_calls: 15_000,
            fps: 60.0,
            ..RenderStats::new()
        };
        assert!(stats_warning(&stats).unwrap().contains("draw call"));
    }

    #[test]
    fn stats_warning_high_frame_time() {
        let stats = RenderStats {
            frame_time_ms: 40.0,
            fps: 60.0,
            ..RenderStats::new()
        };
        assert!(stats_warning(&stats).unwrap().contains("frame time"));
    }

    #[test]
    fn stats_warning_none_when_ok() {
        let stats = RenderStats {
            fps: 60.0,
            frame_time_ms: 16.0,
            draw_calls: 100,
            ..RenderStats::new()
        };
        assert!(stats_warning(&stats).is_none());
    }
}
