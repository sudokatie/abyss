//! Player oxygen supply with depth-based drain.

/// Base oxygen capacity (no tanks).
pub const OXYGEN_BAR_BASE: f32 = 100.0;

/// Oxygen drain rate at surface (units per second).
pub const OXYGEN_DRAIN_RATE: f32 = 1.0;

/// Depth at which drain rate doubles (meters).
pub const OXYGEN_DEPTH_SCALE: f32 = 200.0;

/// Maximum drain rate multiplier (5x at extreme depth).
pub const OXYGEN_MAX_DRAIN_MULTIPLIER: f32 = 5.0;

/// Damage per second when oxygen is depleted.
pub const OXYGEN_DROWNING_DAMAGE: f32 = 5.0;

/// Threshold below which oxygen bar flashes.
pub const OXYGEN_LOW_THRESHOLD: f32 = 0.25;

/// Refill percentage at surface (100%).
pub const REFILL_SURFACE: f32 = 1.0;

/// Refill percentage at air pockets (30%).
pub const REFILL_AIR_POCKET: f32 = 0.30;

/// Refill percentage from oxygen plants (10%).
pub const REFILL_OXYGEN_PLANT: f32 = 0.10;

/// Source of oxygen refill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OxygenRefillSource {
    /// Surface - full refill.
    Surface,
    /// Air pocket in cave - partial refill.
    AirPocket,
    /// Oxygen plant - small refill.
    OxygenPlant,
}

impl OxygenRefillSource {
    /// Get the refill fraction for this source.
    #[must_use]
    pub fn refill_fraction(self) -> f32 {
        match self {
            Self::Surface => REFILL_SURFACE,
            Self::AirPocket => REFILL_AIR_POCKET,
            Self::OxygenPlant => REFILL_OXYGEN_PLANT,
        }
    }
}

/// Player oxygen supply manager.
#[derive(Debug, Clone)]
pub struct OxygenSupply {
    /// Current oxygen level.
    current: f32,
    /// Maximum oxygen capacity (base + tanks).
    max: f32,
    /// External drain rate modifier from equipment (1.0 = normal, 0.5 = rebreather).
    drain_modifier: f32,
    /// Whether the player is currently underwater.
    underwater: bool,
}

impl OxygenSupply {
    /// Create a new oxygen supply with base capacity.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current: OXYGEN_BAR_BASE,
            max: OXYGEN_BAR_BASE,
            drain_modifier: 1.0,
            underwater: false,
        }
    }

    /// Get current oxygen level.
    #[must_use]
    pub fn current(&self) -> f32 {
        self.current
    }

    /// Get maximum oxygen capacity.
    #[must_use]
    pub fn max(&self) -> f32 {
        self.max
    }

    /// Get oxygen as a fraction (0.0 to 1.0).
    #[must_use]
    pub fn fraction(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            self.current / self.max
        }
    }

    /// Check if oxygen is below low threshold.
    #[must_use]
    pub fn is_low(&self) -> bool {
        self.fraction() < OXYGEN_LOW_THRESHOLD
    }

    /// Check if oxygen is depleted.
    #[must_use]
    pub fn is_depleted(&self) -> bool {
        self.current <= 0.0
    }

    /// Set whether the player is underwater.
    pub fn set_underwater(&mut self, underwater: bool) {
        self.underwater = underwater;
    }

    /// Check if player is underwater.
    #[must_use]
    pub fn is_underwater(&self) -> bool {
        self.underwater
    }

    /// Set the drain rate modifier from equipment.
    ///
    /// - 1.0 = normal
    /// - 0.5 = rebreather
    /// - 0.25 = SCUBA
    pub fn set_drain_modifier(&mut self, modifier: f32) {
        self.drain_modifier = modifier.clamp(0.1, 1.0);
    }

    /// Get current drain rate modifier.
    #[must_use]
    pub fn drain_modifier(&self) -> f32 {
        self.drain_modifier
    }

    /// Calculate drain rate at a given depth.
    ///
    /// Uses formula: drain_rate = base * (1.0 + depth / scale) * modifier
    /// Capped at OXYGEN_MAX_DRAIN_MULTIPLIER.
    #[must_use]
    pub fn drain_rate_at_depth(&self, depth: f32) -> f32 {
        let depth_multiplier = 1.0 + (depth / OXYGEN_DEPTH_SCALE);
        let capped_multiplier = depth_multiplier.min(OXYGEN_MAX_DRAIN_MULTIPLIER);
        OXYGEN_DRAIN_RATE * capped_multiplier * self.drain_modifier
    }

    /// Drain oxygen for one tick.
    ///
    /// Returns drowning damage if oxygen is depleted.
    pub fn drain(&mut self, depth: f32, delta_seconds: f32) -> f32 {
        if !self.underwater {
            // Not underwater - slowly refill
            self.refill_partial(0.2 * delta_seconds);
            return 0.0;
        }

        let drain_amount = self.drain_rate_at_depth(depth) * delta_seconds;
        self.current = (self.current - drain_amount).max(0.0);

        if self.is_depleted() {
            OXYGEN_DROWNING_DAMAGE * delta_seconds
        } else {
            0.0
        }
    }

    /// Refill oxygen from a source.
    pub fn refill(&mut self, source: OxygenRefillSource) {
        let amount = self.max * source.refill_fraction();
        self.refill_partial(amount);
    }

    /// Refill by a specific amount.
    pub fn refill_partial(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// Add capacity from an oxygen tank.
    pub fn add_tank_capacity(&mut self, capacity: f32) {
        self.max += capacity;
        // Also fill the new capacity
        self.current += capacity;
    }

    /// Remove capacity from a tank.
    pub fn remove_tank_capacity(&mut self, capacity: f32) {
        self.max = (self.max - capacity).max(OXYGEN_BAR_BASE);
        self.current = self.current.min(self.max);
    }

    /// Get estimated seconds of oxygen remaining at current depth.
    #[must_use]
    pub fn seconds_remaining(&self, depth: f32) -> f32 {
        let rate = self.drain_rate_at_depth(depth);
        if rate <= 0.0 {
            f32::INFINITY
        } else {
            self.current / rate
        }
    }

    /// Reset to full oxygen (debug/respawn).
    pub fn reset(&mut self) {
        self.current = self.max;
    }
}

impl Default for OxygenSupply {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_new_supply() {
        let supply = OxygenSupply::new();
        assert_relative_eq!(supply.current(), OXYGEN_BAR_BASE);
        assert_relative_eq!(supply.max(), OXYGEN_BAR_BASE);
        assert_relative_eq!(supply.fraction(), 1.0);
    }

    #[test]
    fn test_drain_at_surface() {
        let mut supply = OxygenSupply::new();
        supply.set_underwater(true);
        let damage = supply.drain(0.0, 1.0);
        assert_relative_eq!(damage, 0.0);
        assert_relative_eq!(supply.current(), OXYGEN_BAR_BASE - OXYGEN_DRAIN_RATE);
    }

    #[test]
    fn test_drain_faster_at_depth() {
        let mut supply = OxygenSupply::new();
        supply.set_underwater(true);
        supply.drain(200.0, 1.0);
        // At 200m: rate = 1.0 * (1 + 200/200) * 1.0 = 2.0
        assert_relative_eq!(supply.current(), OXYGEN_BAR_BASE - 2.0);
    }

    #[test]
    fn test_drain_capped_at_max() {
        let mut supply = OxygenSupply::new();
        supply.set_underwater(true);
        supply.drain(2000.0, 1.0);
        // At 2000m: uncapped would be 11x, capped at 5x
        let expected_drain = OXYGEN_DRAIN_RATE * OXYGEN_MAX_DRAIN_MULTIPLIER;
        assert_relative_eq!(supply.current(), OXYGEN_BAR_BASE - expected_drain);
    }

    #[test]
    fn test_drowning_damage() {
        let mut supply = OxygenSupply::new();
        supply.set_underwater(true);
        supply.current = 0.5; // Nearly depleted
        let damage = supply.drain(0.0, 1.0);
        assert!(damage > 0.0);
        assert!(supply.is_depleted());
    }

    #[test]
    fn test_no_drain_on_surface() {
        let mut supply = OxygenSupply::new();
        supply.set_underwater(false);
        supply.current = 50.0;
        supply.drain(0.0, 1.0);
        // Should slowly refill
        assert!(supply.current() > 50.0);
    }

    #[test]
    fn test_refill_surface() {
        let mut supply = OxygenSupply::new();
        supply.current = 10.0;
        supply.refill(OxygenRefillSource::Surface);
        assert_relative_eq!(supply.current(), supply.max());
    }

    #[test]
    fn test_refill_air_pocket() {
        let mut supply = OxygenSupply::new();
        supply.current = 10.0;
        let expected = 10.0 + supply.max() * 0.30;
        supply.refill(OxygenRefillSource::AirPocket);
        assert_relative_eq!(supply.current(), expected);
    }

    #[test]
    fn test_refill_oxygen_plant() {
        let mut supply = OxygenSupply::new();
        supply.current = 10.0;
        let expected = 10.0 + supply.max() * 0.10;
        supply.refill(OxygenRefillSource::OxygenPlant);
        assert_relative_eq!(supply.current(), expected);
    }

    #[test]
    fn test_rebreather_modifier() {
        let mut supply = OxygenSupply::new();
        supply.set_drain_modifier(0.5); // Rebreather
        supply.set_underwater(true);
        supply.drain(0.0, 1.0);
        // Drain rate = 1.0 * 1.0 * 0.5 = 0.5
        assert_relative_eq!(supply.current(), OXYGEN_BAR_BASE - 0.5);
    }

    #[test]
    fn test_scuba_modifier() {
        let mut supply = OxygenSupply::new();
        supply.set_drain_modifier(0.25); // SCUBA
        supply.set_underwater(true);
        supply.drain(0.0, 1.0);
        // Drain rate = 1.0 * 1.0 * 0.25 = 0.25
        assert_relative_eq!(supply.current(), OXYGEN_BAR_BASE - 0.25);
    }

    #[test]
    fn test_add_tank_capacity() {
        let mut supply = OxygenSupply::new();
        supply.add_tank_capacity(50.0);
        assert_relative_eq!(supply.max(), 150.0);
        assert_relative_eq!(supply.current(), 150.0); // New capacity also filled
    }

    #[test]
    fn test_remove_tank_capacity() {
        let mut supply = OxygenSupply::new();
        supply.add_tank_capacity(50.0);
        supply.remove_tank_capacity(50.0);
        assert_relative_eq!(supply.max(), OXYGEN_BAR_BASE);
    }

    #[test]
    fn test_low_threshold() {
        let mut supply = OxygenSupply::new();
        assert!(!supply.is_low());
        supply.current = 20.0; // 20% of 100
        assert!(supply.is_low());
    }

    #[test]
    fn test_seconds_remaining() {
        let supply = OxygenSupply::new();
        // At surface: 100 / 1.0 = 100 seconds
        assert_relative_eq!(supply.seconds_remaining(0.0), 100.0);
    }

    #[test]
    fn test_seconds_remaining_at_depth() {
        let supply = OxygenSupply::new();
        // At 200m: rate = 2.0, remaining = 100 / 2.0 = 50 seconds
        assert_relative_eq!(supply.seconds_remaining(200.0), 50.0);
    }

    #[test]
    fn test_drain_modifier_clamped() {
        let mut supply = OxygenSupply::new();
        supply.set_drain_modifier(0.01); // Too low
        assert_relative_eq!(supply.drain_modifier(), 0.1); // Clamped
        supply.set_drain_modifier(5.0); // Too high
        assert_relative_eq!(supply.drain_modifier(), 1.0); // Clamped
    }

    #[test]
    fn test_refill_does_not_exceed_max() {
        let mut supply = OxygenSupply::new();
        supply.refill(OxygenRefillSource::Surface);
        assert_relative_eq!(supply.current(), supply.max());
    }
}
