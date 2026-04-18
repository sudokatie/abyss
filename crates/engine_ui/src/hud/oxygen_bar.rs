//! Oxygen bar HUD element.
//!
//! Displays below health bar, flashes when low.

/// Oxygen bar display state.
#[derive(Debug, Clone)]
pub struct OxygenBar {
    /// Current oxygen (0-100).
    pub oxygen: f32,
    /// Maximum oxygen capacity.
    pub max_oxygen: f32,
    /// Flash timer for low-oxygen warning.
    pub flash_timer: f32,
    /// Whether bar is flashing.
    pub flashing: bool,
}

/// Threshold to start flashing (percentage).
pub const LOW_OXYGEN_THRESHOLD: f32 = 25.0;

/// Flash speed (cycles per second).
pub const FLASH_SPEED: f32 = 3.0;

impl OxygenBar {
    /// Create a new oxygen bar.
    #[must_use]
    pub fn new(max_oxygen: f32) -> Self {
        Self {
            oxygen: max_oxygen,
            max_oxygen,
            flash_timer: 0.0,
            flashing: false,
        }
    }

    /// Update oxygen level.
    pub fn update(&mut self, oxygen: f32, delta: f32) {
        self.oxygen = oxygen;
        self.flashing = self.oxygen_percentage() <= LOW_OXYGEN_THRESHOLD;
        if self.flashing {
            self.flash_timer += delta * FLASH_SPEED;
        } else {
            self.flash_timer = 0.0;
        }
    }

    /// Get oxygen percentage (0-100).
    #[must_use]
    pub fn oxygen_percentage(&self) -> f32 {
        if self.max_oxygen > 0.0 {
            (self.oxygen / self.max_oxygen) * 100.0
        } else {
            0.0
        }
    }

    /// Get bar fill fraction (0.0 to 1.0).
    #[must_use]
    pub fn fill_fraction(&self) -> f32 {
        if self.max_oxygen > 0.0 {
            (self.oxygen / self.max_oxygen).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Get display color (blue when OK, red when low).
    #[must_use]
    pub fn bar_color(&self) -> [f32; 3] {
        if self.oxygen_percentage() <= LOW_OXYGEN_THRESHOLD {
            [1.0, 0.2, 0.2] // Red
        } else if self.oxygen_percentage() <= 50.0 {
            [1.0, 0.8, 0.2] // Yellow
        } else {
            [0.2, 0.6, 1.0] // Blue
        }
    }

    /// Whether the flash is currently visible (for blinking).
    #[must_use]
    pub fn is_flash_visible(&self) -> bool {
        if !self.flashing {
            return true;
        }
        (self.flash_timer * std::f32::consts::PI).sin() > 0.0
    }

    /// Formatted oxygen text.
    #[must_use]
    pub fn oxygen_text(&self) -> String {
        format!("{:.0}/{:.0}", self.oxygen, self.max_oxygen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oxygen_bar_percentage() {
        let bar = OxygenBar::new(100.0);
        assert_relative_eq!(bar.oxygen_percentage(), 100.0);
    }

    #[test]
    fn test_oxygen_bar_flashing() {
        let mut bar = OxygenBar::new(100.0);
        bar.update(20.0, 1.0);
        assert!(bar.flashing);
    }

    #[test]
    fn test_oxygen_bar_color() {
        let mut bar = OxygenBar::new(100.0);
        bar.update(80.0, 0.0);
        assert_relative_eq!(bar.bar_color()[0], 0.2); // Blue
        bar.update(15.0, 0.0);
        assert_relative_eq!(bar.bar_color()[0], 1.0); // Red
    }

    use approx::assert_relative_eq;
}
