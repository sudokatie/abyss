//! Underwater base building.
//!
//! Sealed compartments, air generation, structural integrity,
//! pressure windows, and breach simulation.

/// Default structural integrity (0-100%).
pub const BASE_INTEGRITY_FULL: f32 = 100.0;

/// Integrity threshold for breach warning.
pub const INTEGRITY_WARNING: f32 = 50.0;

/// Integrity threshold for breach (flood).
pub const INTEGRITY_BREACH: f32 = 25.0;

/// Integrity threshold for collapse.
pub const INTEGRITY_COLLAPSE: f32 = 10.0;

/// Air generation rate per oxygen separator (units/sec).
pub const AIR_GENERATION_RATE: f32 = 1.0;

/// Base compartment.
#[derive(Debug, Clone)]
pub struct BaseCompartment {
    /// World position of compartment center.
    pub position: [f32; 3],
    /// Size (blocks) - x, y, z.
    pub size: [f32; 3],
    /// Structural integrity (0-100).
    pub integrity: f32,
    /// Whether compartment is sealed (airtight).
    pub sealed: bool,
    /// Current air level (0-100%).
    pub air_level: f32,
    /// Whether compartment is flooded.
    pub flooded: bool,
    /// Has oxygen separator.
    pub has_oxygen_separator: bool,
    /// Has power.
    pub has_power: bool,
}

impl BaseCompartment {
    /// Create a new base compartment.
    #[must_use]
    pub fn new(position: [f32; 3], size: [f32; 3]) -> Self {
        Self {
            position,
            size,
            integrity: BASE_INTEGRITY_FULL,
            sealed: true,
            air_level: 100.0,
            flooded: false,
            has_oxygen_separator: false,
            has_power: false,
        }
    }

    /// Current depth.
    #[must_use]
    pub fn depth(&self) -> f32 {
        -self.position[1]
    }

    /// Apply depth pressure damage.
    pub fn apply_pressure(&mut self, delta: f32) {
        let depth = self.depth();
        // Deeper bases take more pressure
        let pressure_damage = (depth / 500.0).min(1.0) * 0.5 * delta;
        self.integrity = (self.integrity - pressure_damage).max(0.0);

        // Check for breach
        if self.integrity <= INTEGRITY_BREACH && self.sealed && !self.flooded {
            self.breach();
        }
    }

    /// Simulate a breach - flood the compartment.
    pub fn breach(&mut self) {
        self.flooded = true;
        self.sealed = false;
        self.air_level = 0.0;
    }

    /// Repair integrity.
    pub fn repair(&mut self, amount: f32) {
        self.integrity = (self.integrity + amount).min(BASE_INTEGRITY_FULL);
        if self.integrity > INTEGRITY_BREACH && self.flooded {
            // Can reseal if integrity restored
            self.flooded = false;
            self.sealed = true;
        }
    }

    /// Generate air if powered and sealed.
    pub fn generate_air(&mut self, delta: f32) {
        if self.has_oxygen_separator && self.has_power && self.sealed && !self.flooded {
            self.air_level = (self.air_level + AIR_GENERATION_RATE * delta).min(100.0);
        }
    }

    /// Consume air (player breathing).
    pub fn consume_air(&mut self, rate: f32, delta: f32) {
        if self.sealed && !self.flooded {
            self.air_level = (self.air_level - rate * delta).max(0.0);
        }
    }

    /// Check if integrity is at warning level.
    #[must_use]
    pub fn is_warning(&self) -> bool {
        self.integrity <= INTEGRITY_WARNING
    }

    /// Check if compartment is about to collapse.
    #[must_use]
    pub fn is_collapsing(&self) -> bool {
        self.integrity <= INTEGRITY_COLLAPSE
    }

    /// Check if safe to be inside (sealed, air, not flooded).
    #[must_use]
    pub fn is_safe(&self) -> bool {
        self.sealed && !self.flooded && self.air_level > 0.0
    }

    /// Update compartment state.
    pub fn update(&mut self, delta: f32) {
        self.apply_pressure(delta);
        self.generate_air(delta);
    }
}

/// Pressure window on a compartment wall.
#[derive(Debug, Clone)]
pub struct PressureWindow {
    /// Position on the compartment.
    pub position: [f32; 3],
    /// Window integrity (0-100).
    pub integrity: f32,
    /// Whether the window has cracked.
    pub cracked: bool,
}

impl PressureWindow {
    /// Create a new pressure window.
    #[must_use]
    pub fn new(position: [f32; 3]) -> Self {
        Self {
            position,
            integrity: 100.0,
            cracked: false,
        }
    }

    /// Apply pressure to window.
    pub fn apply_pressure(&mut self, depth: f32, delta: f32) {
        // Windows are more fragile than walls
        let damage = (depth / 300.0).min(1.0) * 1.0 * delta;
        self.integrity = (self.integrity - damage).max(0.0);

        if self.integrity < 50.0 && !self.cracked {
            self.cracked = true;
        }
    }

    /// Check if window has shattered.
    #[must_use]
    pub fn is_shattered(&self) -> bool {
        self.integrity <= 0.0
    }

    /// Repair window.
    pub fn repair(&mut self, amount: f32) {
        self.integrity = (self.integrity + amount).min(100.0);
        if self.integrity > 50.0 {
            self.cracked = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compartment_creation() {
        let comp = BaseCompartment::new([0.0, -200.0, 0.0], [10.0, 5.0, 10.0]);
        assert!(comp.is_safe());
        assert_relative_eq!(comp.integrity, 100.0);
    }

    #[test]
    fn test_pressure_damage() {
        let mut comp = BaseCompartment::new([0.0, -500.0, 0.0], [10.0, 5.0, 10.0]);
        comp.apply_pressure(100.0);
        assert!(comp.integrity < 100.0);
    }

    #[test]
    fn test_breach_floods() {
        let mut comp = BaseCompartment::new([0.0, -500.0, 0.0], [10.0, 5.0, 10.0]);
        comp.integrity = 20.0;
        comp.apply_pressure(1.0);
        assert!(comp.flooded);
    }

    #[test]
    fn test_repair_restores_seal() {
        let mut comp = BaseCompartment::new([0.0, -500.0, 0.0], [10.0, 5.0, 10.0]);
        comp.breach();
        assert!(comp.flooded);
        comp.repair(80.0);
        assert!(!comp.flooded);
        assert!(comp.sealed);
    }

    #[test]
    fn test_air_generation() {
        let mut comp = BaseCompartment::new([0.0, -200.0, 0.0], [10.0, 5.0, 10.0]);
        comp.air_level = 50.0;
        comp.has_oxygen_separator = true;
        comp.has_power = true;
        comp.generate_air(10.0);
        assert!(comp.air_level > 50.0);
    }

    #[test]
    fn test_no_air_without_power() {
        let mut comp = BaseCompartment::new([0.0, -200.0, 0.0], [10.0, 5.0, 10.0]);
        comp.air_level = 50.0;
        comp.has_oxygen_separator = true;
        comp.has_power = false;
        comp.generate_air(10.0);
        assert_relative_eq!(comp.air_level, 50.0);
    }

    #[test]
    fn test_window_cracks() {
        let mut window = PressureWindow::new([0.0, -300.0, 0.0]);
        window.apply_pressure(300.0, 60.0);
        assert!(window.cracked);
    }

    #[test]
    fn test_window_shatters() {
        let mut window = PressureWindow::new([0.0, -300.0, 0.0]);
        window.integrity = 5.0;
        window.apply_pressure(300.0, 10.0);
        assert!(window.is_shattered());
    }

    #[test]
    fn test_warning_level() {
        let comp = BaseCompartment::new([0.0, -200.0, 0.0], [10.0, 5.0, 10.0]);
        assert!(!comp.is_warning());
        let mut damaged = comp;
        damaged.integrity = 40.0;
        assert!(damaged.is_warning());
    }

    use approx::assert_relative_eq;
}
