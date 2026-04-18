//! Light meter HUD element.
//!
//! Shows ambient light level and equipment light battery.

/// Light meter state.
#[derive(Debug, Clone)]
pub struct LightMeter {
    /// Ambient light level (0-1).
    pub ambient_light: f32,
    /// Equipment light radius (blocks, 0 if no light).
    pub light_radius: f32,
    /// Battery level (0-100).
    pub battery: f32,
    /// Whether battery is low.
    pub battery_low: bool,
}

/// Battery low threshold.
pub const BATTERY_LOW_THRESHOLD: f32 = 20.0;

/// Battery drain rate (units/sec).
pub const BATTERY_DRAIN_RATE: f32 = 2.0;

impl LightMeter {
    /// Create a new light meter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ambient_light: 1.0,
            light_radius: 0.0,
            battery: 100.0,
            battery_low: false,
        }
    }

    /// Update light meter.
    pub fn update(&mut self, ambient: f32, light_radius: f32, delta: f32) {
        self.ambient_light = ambient;
        self.light_radius = light_radius;

        // Drain battery when light is on
        if light_radius > 0.0 {
            self.battery = (self.battery - BATTERY_DRAIN_RATE * delta).max(0.0);
        }

        self.battery_low = self.battery <= BATTERY_LOW_THRESHOLD;
    }

    /// Whether it's dark (need light source).
    #[must_use]
    pub fn is_dark(&self) -> bool {
        self.ambient_light < 0.1
    }

    /// Whether the light is currently functional.
    #[must_use]
    pub fn light_functional(&self) -> bool {
        self.light_radius > 0.0 && self.battery > 0.0
    }

    /// Formatted battery text.
    #[must_use]
    pub fn battery_text(&self) -> String {
        format!("Battery: {:.0}%", self.battery)
    }

    /// Formatted light level text.
    #[must_use]
    pub fn light_text(&self) -> String {
        if self.ambient_light < 0.01 {
            "DARK".to_string()
        } else if self.ambient_light < 0.1 {
            "Dim".to_string()
        } else if self.ambient_light < 0.5 {
            "Low Light".to_string()
        } else {
            "Bright".to_string()
        }
    }

    /// Recharge battery.
    pub fn recharge(&mut self, amount: f32) {
        self.battery = (self.battery + amount).min(100.0);
    }
}

impl Default for LightMeter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_meter_drain() {
        let mut meter = LightMeter::new();
        meter.update(0.5, 10.0, 5.0);
        assert!(meter.battery < 100.0);
    }

    #[test]
    fn test_light_meter_dark() {
        let mut meter = LightMeter::new();
        meter.update(0.01, 0.0, 0.0);
        assert!(meter.is_dark());
    }

    #[test]
    fn test_light_meter_recharge() {
        let mut meter = LightMeter::new();
        meter.battery = 10.0;
        meter.recharge(50.0);
        assert_relative_eq!(meter.battery, 60.0);
    }

    use approx::assert_relative_eq;
}
