//! Pressure indicator HUD element.
//!
//! Shows current pressure level and suit integrity.

/// Pressure indicator state.
#[derive(Debug, Clone)]
pub struct PressureIndicator {
    /// Current depth in meters.
    pub depth: f32,
    /// Pressure damage rate (damage/sec).
    pub pressure_damage_rate: f32,
    /// Suit integrity (0-100).
    pub suit_integrity: f32,
    /// Whether pressure is causing damage.
    pub taking_damage: bool,
}

impl PressureIndicator {
    /// Create a new pressure indicator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            depth: 0.0,
            pressure_damage_rate: 0.0,
            suit_integrity: 100.0,
            taking_damage: false,
        }
    }

    /// Update indicator state.
    pub fn update(&mut self, depth: f32, damage_rate: f32, suit_integrity: f32) {
        self.depth = depth;
        self.pressure_damage_rate = damage_rate;
        self.suit_integrity = suit_integrity;
        self.taking_damage = damage_rate > 0.0;
    }

    /// Get indicator color based on pressure state.
    #[must_use]
    pub fn indicator_color(&self) -> [f32; 3] {
        if self.taking_damage {
            [1.0, 0.0, 0.0] // Red - taking damage
        } else if self.suit_integrity < 50.0 {
            [1.0, 0.7, 0.0] // Orange - suit compromised
        } else if self.depth > 200.0 {
            [1.0, 1.0, 0.0] // Yellow - deep
        } else {
            [0.0, 1.0, 0.0] // Green - safe
        }
    }

    /// Get pressure warning text.
    #[must_use]
    pub fn warning_text(&self) -> Option<&'static str> {
        if self.taking_damage {
            Some("PRESSURE DAMAGE")
        } else if self.suit_integrity < 50.0 {
            Some("SUIT COMPROMISED")
        } else if self.depth > 200.0 {
            Some("HIGH PRESSURE")
        } else {
            None
        }
    }

    /// Formatted depth text.
    #[must_use]
    pub fn depth_text(&self) -> String {
        format!("Pressure: {:.0}m", self.depth)
    }
}

impl Default for PressureIndicator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pressure_safe() {
        let mut ind = PressureIndicator::new();
        ind.update(30.0, 0.0, 100.0);
        assert!(!ind.taking_damage);
        assert!(ind.warning_text().is_none());
    }

    #[test]
    fn test_pressure_damage() {
        let mut ind = PressureIndicator::new();
        ind.update(300.0, 5.0, 80.0);
        assert!(ind.taking_damage);
        assert_eq!(ind.warning_text(), Some("PRESSURE DAMAGE"));
    }

    #[test]
    fn test_suit_compromised() {
        let mut ind = PressureIndicator::new();
        ind.update(300.0, 0.0, 30.0);
        assert!(!ind.taking_damage);
        assert_eq!(ind.warning_text(), Some("SUIT COMPROMISED"));
    }
}
