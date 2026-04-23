//! Achievement / advancement toast notification system.
//!
//! Provides [`Toast`] notifications that slide in, display briefly, and fade out,
//! plus a [`ToastQueue`] that manages pending and active toasts with a configurable
//! maximum number of simultaneously visible notifications.

/// How long a toast stays on screen (seconds).
pub const TOAST_DURATION: f32 = 5.0;

/// Fade-in duration in seconds (0.0 .. 0.5).
const FADE_IN_END: f32 = 0.5;
/// Fade-out start time in seconds.
const FADE_OUT_START: f32 = 4.0;

/// The category of a toast notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    /// A standard advancement.
    Advancement,
    /// A goal milestone.
    Goal,
    /// A challenge completion.
    Challenge,
}

/// A single toast notification.
#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    /// The title line shown in the toast.
    pub title: String,
    /// The description line below the title.
    pub description: String,
    /// Item/block icon identifier.
    pub icon_id: u16,
    /// Elapsed time since this toast became active (seconds).
    pub timer: f32,
    /// The category of this toast.
    pub toast_type: ToastType,
}

impl Toast {
    /// Create a new toast with `timer` initialized to `0.0`.
    pub fn new(title: String, description: String, icon_id: u16, toast_type: ToastType) -> Self {
        Self {
            title,
            description,
            icon_id,
            timer: 0.0,
            toast_type,
        }
    }
}

/// A queue that manages pending and active toasts.
#[derive(Debug, Clone)]
pub struct ToastQueue {
    /// Currently visible toasts.
    pub active: Vec<Toast>,
    /// Toasts waiting to be shown.
    pub pending: Vec<Toast>,
    /// Maximum number of simultaneously visible toasts.
    pub max_visible: usize,
}

impl ToastQueue {
    /// Create an empty queue with `max_visible` set to `1`.
    pub fn new() -> Self {
        Self {
            active: Vec::new(),
            pending: Vec::new(),
            max_visible: 1,
        }
    }

    /// Enqueue a toast for display. It will appear once there is room.
    pub fn push(&mut self, toast: Toast) {
        self.pending.push(toast);
    }

    /// Advance all active toast timers by `dt` seconds, remove expired ones, and
    /// promote pending toasts into the active set when space is available.
    pub fn tick(&mut self, dt: f32) {
        // Advance timers on active toasts.
        for toast in &mut self.active {
            toast.timer += dt;
        }

        // Remove expired toasts.
        self.active.retain(|t| t.timer < TOAST_DURATION);

        // Promote from pending while there is room.
        while self.active.len() < self.max_visible {
            if let Some(toast) = self.pending.first() {
                // Ensure the toast starts fresh.
                debug_assert!(
                    toast.timer == 0.0,
                    "pending toast should have timer == 0.0"
                );
                let toast = self.pending.remove(0);
                self.active.push(toast);
            } else {
                break;
            }
        }
    }

    /// Return a reference to the toast that should currently be rendered, if any.
    pub fn current_toast(&self) -> Option<&Toast> {
        self.active.first()
    }
}

impl Default for ToastQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute the alpha (opacity) for a toast given its elapsed `timer`.
///
/// - `0.0 ..  0.5` s: fade in from `0.0` to `1.0`
/// - `0.5 ..  4.0` s: fully opaque (`1.0`)
/// - `4.0 ..  5.0` s: fade out from `1.0` to `0.0`
/// - `>= 5.0`     s: `0.0`
pub fn toast_alpha(timer: f32) -> f32 {
    if timer < 0.0 {
        0.0
    } else if timer < FADE_IN_END {
        timer / FADE_IN_END
    } else if timer < FADE_OUT_START {
        1.0
    } else if timer < TOAST_DURATION {
        1.0 - (timer - FADE_OUT_START) / (TOAST_DURATION - FADE_OUT_START)
    } else {
        0.0
    }
}

/// Return the human-readable prefix for a given [`ToastType`].
pub fn toast_type_prefix(t: ToastType) -> &'static str {
    match t {
        ToastType::Advancement => "Advancement Made!",
        ToastType::Goal => "Goal Reached!",
        ToastType::Challenge => "Challenge Complete!",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Toast construction ───────────────────────────────────────────

    #[test]
    fn new_toast_has_zero_timer() {
        let t = Toast::new(
            "Title".into(),
            "Desc".into(),
            42,
            ToastType::Advancement,
        );
        assert_eq!(t.timer, 0.0);
        assert_eq!(t.title, "Title");
        assert_eq!(t.description, "Desc");
        assert_eq!(t.icon_id, 42);
        assert_eq!(t.toast_type, ToastType::Advancement);
    }

    // ── ToastQueue push / tick lifecycle ─────────────────────────────

    #[test]
    fn push_adds_to_pending() {
        let mut q = ToastQueue::new();
        q.push(Toast::new("A".into(), "".into(), 0, ToastType::Goal));
        assert_eq!(q.pending.len(), 1);
        assert!(q.active.is_empty());
    }

    #[test]
    fn tick_promotes_pending_to_active() {
        let mut q = ToastQueue::new();
        q.push(Toast::new("A".into(), "".into(), 0, ToastType::Goal));
        q.tick(0.0);
        assert_eq!(q.active.len(), 1);
        assert!(q.pending.is_empty());
    }

    #[test]
    fn tick_advances_timer() {
        let mut q = ToastQueue::new();
        q.push(Toast::new("A".into(), "".into(), 0, ToastType::Goal));
        q.tick(0.0); // promote
        q.tick(1.0); // advance
        assert!((q.active[0].timer - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_removes_expired_toast() {
        let mut q = ToastQueue::new();
        q.push(Toast::new("A".into(), "".into(), 0, ToastType::Goal));
        q.tick(0.0);
        q.tick(TOAST_DURATION + 0.1);
        assert!(q.active.is_empty());
    }

    #[test]
    fn tick_promotes_next_after_expiry() {
        let mut q = ToastQueue::new();
        q.push(Toast::new("A".into(), "".into(), 0, ToastType::Goal));
        q.push(Toast::new("B".into(), "".into(), 0, ToastType::Challenge));
        q.tick(0.0); // promote A
        q.tick(TOAST_DURATION + 0.1); // expire A, promote B
        assert_eq!(q.active.len(), 1);
        assert_eq!(q.active[0].title, "B");
    }

    #[test]
    fn current_toast_returns_first_active() {
        let mut q = ToastQueue::new();
        assert!(q.current_toast().is_none());
        q.push(Toast::new("A".into(), "".into(), 0, ToastType::Advancement));
        q.tick(0.0);
        assert_eq!(q.current_toast().unwrap().title, "A");
    }

    #[test]
    fn queue_respects_max_visible() {
        let mut q = ToastQueue::new();
        q.max_visible = 2;
        q.push(Toast::new("A".into(), "".into(), 0, ToastType::Goal));
        q.push(Toast::new("B".into(), "".into(), 0, ToastType::Goal));
        q.push(Toast::new("C".into(), "".into(), 0, ToastType::Goal));
        q.tick(0.0);
        assert_eq!(q.active.len(), 2);
        assert_eq!(q.pending.len(), 1);
    }

    #[test]
    fn queue_ordering_is_fifo() {
        let mut q = ToastQueue::new();
        q.max_visible = 3;
        q.push(Toast::new("1".into(), "".into(), 0, ToastType::Goal));
        q.push(Toast::new("2".into(), "".into(), 0, ToastType::Goal));
        q.push(Toast::new("3".into(), "".into(), 0, ToastType::Goal));
        q.tick(0.0);
        let titles: Vec<&str> = q.active.iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, vec!["1", "2", "3"]);
    }

    // ── Fade timing ─────────────────────────────────────────────────

    #[test]
    fn alpha_at_zero_is_zero() {
        assert!((toast_alpha(0.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn alpha_fades_in_linearly() {
        let mid = toast_alpha(0.25);
        assert!((mid - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn alpha_full_during_display() {
        assert!((toast_alpha(0.5) - 1.0).abs() < f32::EPSILON);
        assert!((toast_alpha(2.0) - 1.0).abs() < f32::EPSILON);
        assert!((toast_alpha(3.9) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn alpha_fades_out_linearly() {
        let mid = toast_alpha(4.5);
        assert!((mid - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn alpha_at_duration_is_zero() {
        assert!((toast_alpha(TOAST_DURATION)).abs() < f32::EPSILON);
    }

    #[test]
    fn alpha_beyond_duration_is_zero() {
        assert!((toast_alpha(6.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn alpha_negative_timer_is_zero() {
        assert!((toast_alpha(-1.0)).abs() < f32::EPSILON);
    }

    // ── toast_type_prefix ───────────────────────────────────────────

    #[test]
    fn prefix_advancement() {
        assert_eq!(toast_type_prefix(ToastType::Advancement), "Advancement Made!");
    }

    #[test]
    fn prefix_goal() {
        assert_eq!(toast_type_prefix(ToastType::Goal), "Goal Reached!");
    }

    #[test]
    fn prefix_challenge() {
        assert_eq!(toast_type_prefix(ToastType::Challenge), "Challenge Complete!");
    }
}
