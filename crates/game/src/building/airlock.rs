//! Airlock and power systems for underwater bases.
//!
//! Airlock prevents flooding during entry/exit.
//! Power from thermal, solar, and bio generators.

/// Airlock cycle time (seconds).
pub const AIRLOCK_CYCLE_TIME: f32 = 5.0;

/// Airlock states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AirlockState {
    /// Both doors closed, waiting for input.
    Idle,
    /// Cycling: closing outer door, draining water.
    Cycling,
    /// Ready to open inner door.
    Ready,
    /// Open for passage.
    Open,
}

/// An airlock connecting water to base interior.
#[derive(Debug, Clone)]
pub struct Airlock {
    pub position: [f32; 3],
    pub state: AirlockState,
    pub cycle_timer: f32,
    /// Whether the outer door is open.
    pub outer_door_open: bool,
    /// Whether the inner door is open.
    pub inner_door_open: bool,
    /// Whether water is present in the airlock chamber.
    pub flooded: bool,
}

impl Airlock {
    /// Create a new airlock.
    #[must_use]
    pub fn new(position: [f32; 3]) -> Self {
        Self {
            position,
            state: AirlockState::Idle,
            cycle_timer: 0.0,
            outer_door_open: false,
            inner_door_open: false,
            flooded: true, // Starts flooded (connected to water)
        }
    }

    /// Request to enter base from water.
    pub fn request_entry(&mut self) -> bool {
        if self.state != AirlockState::Idle || self.inner_door_open {
            return false;
        }
        if self.outer_door_open {
            // Close outer door first
            self.outer_door_open = false;
        }
        self.state = AirlockState::Cycling;
        self.cycle_timer = AIRLOCK_CYCLE_TIME;
        true
    }

    /// Request to exit base to water.
    pub fn request_exit(&mut self) -> bool {
        if self.state != AirlockState::Idle || self.outer_door_open {
            return false;
        }
        if self.inner_door_open {
            self.inner_door_open = false;
        }
        self.state = AirlockState::Cycling;
        self.cycle_timer = AIRLOCK_CYCLE_TIME;
        true
    }

    /// Force open both doors (emergency - floods the base).
    pub fn force_open(&mut self) {
        self.outer_door_open = true;
        self.inner_door_open = true;
        self.flooded = true;
        self.state = AirlockState::Open;
    }

    /// Update airlock cycle.
    pub fn update(&mut self, delta: f32) {
        match self.state {
            AirlockState::Cycling => {
                self.cycle_timer -= delta;
                if self.cycle_timer <= 0.0 {
                    self.state = AirlockState::Ready;
                    self.flooded = false;
                    self.cycle_timer = 0.0;
                }
            }
            AirlockState::Ready => {
                // Auto-transition to open
                self.inner_door_open = true;
                self.outer_door_open = false;
                self.flooded = false;
                self.state = AirlockState::Open;
            }
            AirlockState::Open => {
                // Close after a delay (simplified)
            }
            AirlockState::Idle => {}
        }
    }

    /// Reset airlock to idle (after passage).
    pub fn reset(&mut self) {
        self.inner_door_open = false;
        self.outer_door_open = false;
        self.flooded = true;
        self.state = AirlockState::Idle;
    }

    /// Check if cycling is in progress.
    #[must_use]
    pub fn is_cycling(&self) -> bool {
        self.state == AirlockState::Cycling
    }

    /// Check if safe to pass through.
    #[must_use]
    pub fn is_safe(&self) -> bool {
        !self.flooded && self.state == AirlockState::Open
    }
}

/// Power generator type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PowerGenerator {
    /// Thermal generator near vents.
    Thermal,
    /// Solar panels at surface.
    Solar,
    /// Bio generator from organic matter.
    Bio,
}

impl PowerGenerator {
    /// Power output (units/sec).
    #[must_use]
    pub fn output(&self) -> f32 {
        match self {
            PowerGenerator::Thermal => 10.0,
            PowerGenerator::Solar => 5.0,
            PowerGenerator::Bio => 3.0,
        }
    }

    /// Maximum depth for operation.
    #[must_use]
    pub fn max_depth(&self) -> f32 {
        match self {
            PowerGenerator::Thermal => 3000.0,
            PowerGenerator::Solar => 20.0,
            PowerGenerator::Bio => 500.0,
        }
    }

    /// Whether this generator requires a feature nearby.
    #[must_use]
    pub fn requires_feature(&self) -> bool {
        self == &PowerGenerator::Thermal
    }

    /// Check if generator works at depth.
    #[must_use]
    pub fn works_at_depth(&self, depth: f32) -> bool {
        depth <= self.max_depth()
    }
}

/// Power system for a base.
#[derive(Debug, Clone)]
pub struct PowerSystem {
    pub generators: Vec<PowerGenerator>,
    pub stored_power: f32,
    pub max_storage: f32,
    /// Current consumption (units/sec).
    pub consumption: f32,
}

impl PowerSystem {
    /// Create a new power system.
    #[must_use]
    pub fn new(max_storage: f32) -> Self {
        Self {
            generators: Vec::new(),
            stored_power: 0.0,
            max_storage,
            consumption: 0.0,
        }
    }

    /// Add a generator.
    pub fn add_generator(&mut self, generator: PowerGenerator) {
        self.generators.push(generator);
    }

    /// Get total power production.
    #[must_use]
    pub fn production(&self) -> f32 {
        self.generators.iter().map(|g| g.output()).sum()
    }

    /// Get net power (production - consumption).
    #[must_use]
    pub fn net_power(&self) -> f32 {
        self.production() - self.consumption
    }

    /// Check if system has power.
    #[must_use]
    pub fn has_power(&self) -> bool {
        self.stored_power > 0.0 || self.net_power() > 0.0
    }

    /// Update power system.
    pub fn update(&mut self, delta: f32) {
        let net = self.net_power();
        self.stored_power = (self.stored_power + net * delta).clamp(0.0, self.max_storage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_airlock_starts_idle() {
        let airlock = Airlock::new([0.0, -200.0, 0.0]);
        assert_eq!(airlock.state, AirlockState::Idle);
        assert!(airlock.flooded);
    }

    #[test]
    fn test_airlock_cycle() {
        let mut airlock = Airlock::new([0.0, -200.0, 0.0]);
        assert!(airlock.request_entry());
        assert!(airlock.is_cycling());
        airlock.update(AIRLOCK_CYCLE_TIME + 1.0);
        assert!(!airlock.flooded);
    }

    #[test]
    fn test_force_open_floods() {
        let mut airlock = Airlock::new([0.0, -200.0, 0.0]);
        airlock.request_entry();
        airlock.update(AIRLOCK_CYCLE_TIME + 1.0);
        airlock.force_open();
        assert!(airlock.flooded);
    }

    #[test]
    fn test_thermal_output() {
        assert!(PowerGenerator::Thermal.output() > PowerGenerator::Solar.output());
    }

    #[test]
    fn test_solar_shallow_only() {
        assert!(PowerGenerator::Solar.works_at_depth(10.0));
        assert!(!PowerGenerator::Solar.works_at_depth(100.0));
    }

    #[test]
    fn test_power_system() {
        let mut sys = PowerSystem::new(100.0);
        sys.add_generator(PowerGenerator::Thermal);
        sys.add_generator(PowerGenerator::Solar);
        assert!(sys.production() > 0.0);
        sys.update(10.0);
        assert!(sys.stored_power > 0.0);
    }

    #[test]
    fn test_power_consumption() {
        let mut sys = PowerSystem::new(100.0);
        sys.add_generator(PowerGenerator::Bio);
        sys.consumption = 10.0; // More than production
        sys.update(10.0);
        assert_relative_eq!(sys.stored_power, 0.0);
    }

    #[test]
    fn test_airlock_reset() {
        let mut airlock = Airlock::new([0.0, -200.0, 0.0]);
        airlock.request_entry();
        airlock.update(AIRLOCK_CYCLE_TIME + 1.0);
        airlock.reset();
        assert_eq!(airlock.state, AirlockState::Idle);
    }

    use approx::assert_relative_eq;
}
