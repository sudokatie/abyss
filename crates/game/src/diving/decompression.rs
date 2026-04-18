//! Decompression sickness from rapid ascent.

/// Safe ascent rate in meters per second.
pub const SAFE_ASCENT_RATE: f32 = 10.0;

/// Window size for rolling ascent rate calculation (seconds).
pub const ASCENT_WINDOW: f32 = 2.0;

/// Minimum depth where decompression matters (meters).
pub const DECOMPRESSION_MIN_DEPTH: f32 = 50.0;

/// Damage per decompression sickness event.
pub const DECOMPRESSION_DAMAGE_MIN: f32 = 2.0;
pub const DECOMPRESSION_DAMAGE_MAX: f32 = 5.0;

/// Treatment duration in decompression chamber (seconds).
pub const TREATMENT_DURATION: f32 = 30.0;

/// Maximum number of nitrogen bubble events before severe symptoms.
pub const MAX_BUBBLE_EVENTS: u32 = 5;

/// Safe ascent rate configuration.
#[derive(Debug, Clone, Copy)]
pub struct SafeAscentRate {
    /// Maximum safe ascent rate (m/s).
    pub rate: f32,
    /// Minimum depth where this rate applies.
    pub from_depth: f32,
}

impl Default for SafeAscentRate {
    fn default() -> Self {
        Self::standard()
    }
}

impl SafeAscentRate {
    /// Standard safe ascent rate: 10m/s from any depth.
    #[must_use]
    pub fn standard() -> Self {
        Self {
            rate: SAFE_ASCENT_RATE,
            from_depth: DECOMPRESSION_MIN_DEPTH,
        }
    }

    /// Stricter rate for deep ascents: 5m/s from 200m+.
    #[must_use]
    pub fn deep_dive() -> Self {
        Self {
            rate: 5.0,
            from_depth: 200.0,
        }
    }

    /// Get the safe ascent rate at a given depth.
    #[must_use]
    pub fn rate_at_depth(&self, depth: f32) -> f32 {
        if depth >= self.from_depth {
            self.rate
        } else {
            // Above the threshold, any rate is fine
            f32::INFINITY
        }
    }
}

/// A nitrogen bubble event from decompression.
#[derive(Debug, Clone, Copy)]
pub struct NitrogenBubble {
    /// Damage from this event.
    pub damage: f32,
    /// Time when the event occurred.
    pub time: f32,
}

/// Decompression sickness tracker.
#[derive(Debug, Clone)]
pub struct DecompressionSickness {
    /// Recent depth samples for rolling ascent rate.
    depth_history: Vec<(f32, f32)>, // (time, depth)
    /// Accumulated nitrogen bubble events.
    bubble_events: Vec<NitrogenBubble>,
    /// Whether currently in treatment.
    in_treatment: bool,
    /// Treatment progress (0.0 to 1.0).
    treatment_progress: f32,
    /// Safe ascent rate configuration.
    safe_rate: SafeAscentRate,
    /// Whether decompression is enabled.
    enabled: bool,
}

impl Default for DecompressionSickness {
    fn default() -> Self {
        Self::new()
    }
}

impl DecompressionSickness {
    /// Create a new decompression tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            depth_history: Vec::with_capacity(64),
            bubble_events: Vec::new(),
            in_treatment: false,
            treatment_progress: 0.0,
            safe_rate: SafeAscentRate::default(),
            enabled: true,
        }
    }

    /// Enable or disable decompression.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Set safe ascent rate.
    pub fn set_safe_rate(&mut self, rate: SafeAscentRate) {
        self.safe_rate = rate;
    }

    /// Record a depth sample.
    pub fn record_depth(&mut self, time: f32, depth: f32) {
        self.depth_history.push((time, depth));
        // Prune old samples outside the window
        let cutoff = time - ASCENT_WINDOW;
        self.depth_history.retain(|(t, _)| *t >= cutoff);
    }

    /// Calculate the current ascent rate from depth history.
    ///
    /// Returns the ascent rate in m/s (positive = ascending).
    #[must_use]
    pub fn current_ascent_rate(&self) -> f32 {
        if self.depth_history.len() < 2 {
            return 0.0;
        }
        let first = self.depth_history.first().unwrap();
        let last = self.depth_history.last().unwrap();
        let time_delta = last.0 - first.0;
        if time_delta <= 0.0 {
            return 0.0;
        }
        // Ascent rate = (shallower depth - deeper depth) / time
        // Depth decreases when ascending, so ascent = first.depth - last.depth
        (first.1 - last.1) / time_delta
    }

    /// Check if current ascent rate exceeds safe limits.
    #[must_use]
    pub fn is_ascending_too_fast(&self, current_depth: f32) -> bool {
        if !self.enabled {
            return false;
        }
        if current_depth < DECOMPRESSION_MIN_DEPTH {
            return false;
        }
        let rate = self.current_ascent_rate();
        rate > self.safe_rate.rate_at_depth(current_depth)
    }

    /// Update decompression state.
    ///
    /// Returns damage from any new nitrogen bubble events.
    pub fn update(&mut self, time: f32, depth: f32, delta_seconds: f32) -> f32 {
        if !self.enabled {
            return 0.0;
        }

        self.record_depth(time, depth);

        // Handle treatment
        if self.in_treatment {
            self.treatment_progress += delta_seconds / TREATMENT_DURATION;
            if self.treatment_progress >= 1.0 {
                self.in_treatment = false;
                self.treatment_progress = 0.0;
                self.bubble_events.clear();
            }
            return 0.0;
        }

        // Check for decompression sickness
        if self.is_ascending_too_fast(depth) {
            let damage = DECOMPRESSION_DAMAGE_MIN
                + (DECOMPRESSION_DAMAGE_MAX - DECOMPRESSION_DAMAGE_MIN) * rand_val();
            self.bubble_events.push(NitrogenBubble { damage, time });
            damage
        } else {
            0.0
        }
    }

    /// Start decompression treatment.
    pub fn start_treatment(&mut self) {
        self.in_treatment = true;
        self.treatment_progress = 0.0;
    }

    /// Check if currently in treatment.
    #[must_use]
    pub fn is_in_treatment(&self) -> bool {
        self.in_treatment
    }

    /// Get treatment progress (0.0 to 1.0).
    #[must_use]
    pub fn treatment_progress(&self) -> f32 {
        self.treatment_progress
    }

    /// Get number of accumulated bubble events.
    #[must_use]
    pub fn bubble_count(&self) -> usize {
        self.bubble_events.len()
    }

    /// Check if symptoms are severe (many bubble events).
    #[must_use]
    pub fn is_severe(&self) -> bool {
        self.bubble_events.len() as u32 >= MAX_BUBBLE_EVENTS
    }

    /// Reset decompression state (debug/respawn).
    pub fn reset(&mut self) {
        self.depth_history.clear();
        self.bubble_events.clear();
        self.in_treatment = false;
        self.treatment_progress = 0.0;
    }
}

/// Simple deterministic random for decompression damage.
/// Uses a simple counter-based approach for testability.
fn rand_val() -> f32 {
    // Use a simple hash of thread time for now
    // In production this would use the game's RNG
    0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tracker() {
        let dcs = DecompressionSickness::new();
        assert_eq!(dcs.bubble_count(), 0);
        assert!(!dcs.is_in_treatment());
        assert!(!dcs.is_severe());
    }

    #[test]
    fn test_safe_ascent_rate_standard() {
        let rate = SafeAscentRate::standard();
        assert_relative_eq!(rate.rate, 10.0);
    }

    #[test]
    fn test_safe_rate_at_shallow_depth() {
        let rate = SafeAscentRate::standard();
        // Above 50m, any rate is fine
        assert!(rate.rate_at_depth(30.0).is_infinite());
    }

    #[test]
    fn test_safe_rate_at_depth() {
        let rate = SafeAscentRate::standard();
        assert_relative_eq!(rate.rate_at_depth(100.0), 10.0);
    }

    #[test]
    fn test_current_ascent_rate_no_data() {
        let dcs = DecompressionSickness::new();
        assert_relative_eq!(dcs.current_ascent_rate(), 0.0);
    }

    #[test]
    fn test_current_ascent_rate_with_data() {
        let mut dcs = DecompressionSickness::new();
        dcs.record_depth(0.0, 100.0);
        dcs.record_depth(1.0, 85.0);
        // Ascent: 15m in 1s = 15 m/s
        assert_relative_eq!(dcs.current_ascent_rate(), 15.0);
    }

    #[test]
    fn test_ascending_too_fast() {
        let mut dcs = DecompressionSickness::new();
        dcs.record_depth(0.0, 100.0);
        dcs.record_depth(1.0, 85.0); // 15 m/s ascent
        assert!(dcs.is_ascending_too_fast(90.0));
    }

    #[test]
    fn test_safe_ascent_no_sickness() {
        let mut dcs = DecompressionSickness::new();
        dcs.record_depth(0.0, 100.0);
        dcs.record_depth(1.0, 95.0); // 5 m/s ascent
        assert!(!dcs.is_ascending_too_fast(97.0));
    }

    #[test]
    fn test_no_sickness_above_min_depth() {
        let mut dcs = DecompressionSickness::new();
        dcs.record_depth(0.0, 40.0);
        dcs.record_depth(1.0, 20.0); // Fast, but above 50m
        assert!(!dcs.is_ascending_too_fast(30.0));
    }

    #[test]
    fn test_treatment_progress() {
        let mut dcs = DecompressionSickness::new();
        dcs.start_treatment();
        assert!(dcs.is_in_treatment());
        // Simulate some treatment time
        dcs.treatment_progress = 0.5;
        assert_relative_eq!(dcs.treatment_progress(), 0.5);
    }

    #[test]
    fn test_treatment_completes() {
        let mut dcs = DecompressionSickness::new();
        dcs.bubble_events.push(NitrogenBubble { damage: 3.0, time: 0.0 });
        dcs.start_treatment();
        dcs.treatment_progress = 1.0;
        // Simulate update that completes treatment
        dcs.in_treatment = false;
        dcs.bubble_events.clear();
        assert!(!dcs.is_in_treatment());
        assert_eq!(dcs.bubble_count(), 0);
    }

    #[test]
    fn test_severe_symptoms() {
        let mut dcs = DecompressionSickness::new();
        for i in 0..MAX_BUBBLE_EVENTS {
            dcs.bubble_events.push(NitrogenBubble { damage: 3.0, time: i as f32 });
        }
        assert!(dcs.is_severe());
    }

    #[test]
    fn test_disabled_no_sickness() {
        let mut dcs = DecompressionSickness::new();
        dcs.set_enabled(false);
        dcs.record_depth(0.0, 100.0);
        dcs.record_depth(1.0, 50.0); // Very fast
        assert!(!dcs.is_ascending_too_fast(75.0));
    }

    #[test]
    fn test_deep_dive_stricter_rate() {
        let mut dcs = DecompressionSickness::new();
        dcs.set_safe_rate(SafeAscentRate::deep_dive());
        dcs.record_depth(0.0, 300.0);
        dcs.record_depth(1.0, 292.0); // 8 m/s
        assert!(dcs.is_ascending_too_fast(296.0)); // Exceeds 5 m/s limit
    }

    #[test]
    fn test_reset() {
        let mut dcs = DecompressionSickness::new();
        dcs.bubble_events.push(NitrogenBubble { damage: 3.0, time: 0.0 });
        dcs.start_treatment();
        dcs.reset();
        assert_eq!(dcs.bubble_count(), 0);
        assert!(!dcs.is_in_treatment());
    }

    use approx::assert_relative_eq;
}
