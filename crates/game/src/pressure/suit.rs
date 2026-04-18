//! Pressure suit equipment for deep diving.

/// Rated depth for basic pressure suit (meters).
pub const BASIC_SUIT_RATED_DEPTH: f32 = 200.0;

/// Rated depth for reinforced pressure suit (meters).
pub const REINFORCED_SUIT_RATED_DEPTH: f32 = 500.0;

/// Rated depth for pressure vessel (meters).
pub const VESSEL_RATED_DEPTH: f32 = f32::MAX;

/// Protection factor for basic suit (0.8 = 80% damage reduction).
pub const BASIC_SUIT_PROTECTION: f32 = 0.8;

/// Protection factor for reinforced suit.
pub const REINFORCED_SUIT_PROTECTION: f32 = 0.8;

/// Protection factor for pressure vessel.
pub const VESSEL_SUIT_PROTECTION: f32 = 0.95;

/// Integrity loss rate per second at rated depth.
pub const INTEGRITY_LOSS_RATE: f32 = 0.001;

/// Integrity loss multiplier when exceeding rated depth.
pub const OVER_DEPTH_INTEGRITY_MULTIPLIER: f32 = 5.0;

/// Suit tier determining depth rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuitTier {
    /// Basic suit, rated to 200m.
    Basic,
    /// Reinforced suit, rated to 500m.
    Reinforced,
    /// Pressure vessel, rated for any depth.
    Vessel,
}

impl SuitTier {
    /// Get the rated depth for this suit tier.
    #[must_use]
    pub fn rated_depth(self) -> f32 {
        match self {
            Self::Basic => BASIC_SUIT_RATED_DEPTH,
            Self::Reinforced => REINFORCED_SUIT_RATED_DEPTH,
            Self::Vessel => VESSEL_RATED_DEPTH,
        }
    }

    /// Get the protection factor for this suit tier.
    #[must_use]
    pub fn protection(self) -> f32 {
        match self {
            Self::Basic => BASIC_SUIT_PROTECTION,
            Self::Reinforced => REINFORCED_SUIT_PROTECTION,
            Self::Vessel => VESSEL_SUIT_PROTECTION,
        }
    }

    /// Human-readable tier name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Basic => "Basic Pressure Suit",
            Self::Reinforced => "Reinforced Pressure Suit",
            Self::Vessel => "Pressure Vessel",
        }
    }
}

/// A pressure suit that protects against depth damage.
#[derive(Debug, Clone)]
pub struct PressureSuit {
    /// Suit tier.
    tier: SuitTier,
    /// Current integrity (0.0 to 1.0).
    integrity: f32,
    /// Whether the suit is equipped.
    equipped: bool,
}

impl PressureSuit {
    /// Create a new pressure suit of the given tier.
    #[must_use]
    pub fn new(tier: SuitTier) -> Self {
        Self {
            tier,
            integrity: 1.0,
            equipped: true,
        }
    }

    /// Get the suit tier.
    #[must_use]
    pub fn tier(&self) -> SuitTier {
        self.tier
    }

    /// Get current integrity (0.0 to 1.0).
    #[must_use]
    pub fn integrity(&self) -> f32 {
        self.integrity
    }

    /// Check if the suit is equipped.
    #[must_use]
    pub fn is_equipped(&self) -> bool {
        self.equipped
    }

    /// Equip the suit.
    pub fn equip(&mut self) {
        self.equipped = true;
    }

    /// Unequip the suit.
    pub fn unequip(&mut self) {
        self.equipped = false;
    }

    /// Check if suit is functional (integrity > 0).
    #[must_use]
    pub fn is_functional(&self) -> bool {
        self.integrity > 0.0
    }

    /// Check if the suit is at a depth it can handle.
    #[must_use]
    pub fn is_within_rating(&self, depth: f32) -> bool {
        depth <= self.tier.rated_depth()
    }

    /// Update suit integrity based on current depth.
    ///
    /// Integrity degrades faster when near or beyond the suit's rated depth.
    pub fn update_integrity(&mut self, depth: f32, delta_seconds: f32) {
        if !self.equipped {
            return;
        }

        let base_loss = INTEGRITY_LOSS_RATE * delta_seconds;

        let loss = if depth > self.tier.rated_depth() {
            // Beyond rated depth - fast degradation
            let over_ratio = depth / self.tier.rated_depth();
            base_loss * OVER_DEPTH_INTEGRITY_MULTIPLIER * over_ratio
        } else if depth > self.tier.rated_depth() * 0.8 {
            // Near rated depth - moderate degradation
            base_loss * 2.0
        } else {
            // Well within rating - slow degradation
            base_loss
        };

        self.integrity = (self.integrity - loss).max(0.0);
    }

    /// Repair suit integrity by a given amount (0.0 to 1.0).
    pub fn repair(&mut self, amount: f32) {
        self.integrity = (self.integrity + amount).min(1.0);
    }

    /// Fully repair the suit.
    pub fn repair_full(&mut self) {
        self.integrity = 1.0;
    }

    /// Get effective protection at current integrity.
    ///
    /// Protection scales linearly with integrity.
    #[must_use]
    pub fn effective_protection(&self) -> f32 {
        self.tier.protection() * self.integrity
    }

    /// Calculate damage after suit protection.
    #[must_use]
    pub fn apply_protection(&self, base_damage: f32, depth: f32) -> f32 {
        if !self.equipped || !self.is_functional() {
            return base_damage;
        }
        if depth > self.tier.rated_depth() {
            return base_damage; // No protection beyond rating
        }
        (base_damage * (1.0 - self.effective_protection())).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_suit() {
        let suit = PressureSuit::new(SuitTier::Basic);
        assert_eq!(suit.tier(), SuitTier::Basic);
        assert_relative_eq!(suit.integrity(), 1.0);
        assert!(suit.is_equipped());
        assert!(suit.is_functional());
    }

    #[test]
    fn test_rated_depths() {
        assert_relative_eq!(SuitTier::Basic.rated_depth(), 200.0);
        assert_relative_eq!(SuitTier::Reinforced.rated_depth(), 500.0);
        assert!(SuitTier::Vessel.rated_depth() > 10000.0);
    }

    #[test]
    fn test_within_rating() {
        let basic = PressureSuit::new(SuitTier::Basic);
        assert!(basic.is_within_rating(100.0));
        assert!(basic.is_within_rating(200.0));
        assert!(!basic.is_within_rating(201.0));
    }

    #[test]
    fn test_integrity_degradation_at_depth() {
        let mut suit = PressureSuit::new(SuitTier::Basic);
        let start = suit.integrity();
        suit.update_integrity(100.0, 60.0); // 60 seconds at 100m
        assert!(suit.integrity() < start);
    }

    #[test]
    fn test_integrity_degradation_beyond_rating() {
        let mut suit = PressureSuit::new(SuitTier::Basic);
        let start = suit.integrity();
        suit.update_integrity(300.0, 60.0); // Beyond 200m rating
        // Should degrade faster
        let mut suit2 = PressureSuit::new(SuitTier::Basic);
        suit2.update_integrity(100.0, 60.0); // Within rating
        assert!(suit.integrity() < suit2.integrity());
    }

    #[test]
    fn test_repair() {
        let mut suit = PressureSuit::new(SuitTier::Basic);
        suit.integrity = 0.5;
        suit.repair(0.3);
        assert_relative_eq!(suit.integrity(), 0.8);
    }

    #[test]
    fn test_repair_full() {
        let mut suit = PressureSuit::new(SuitTier::Basic);
        suit.integrity = 0.1;
        suit.repair_full();
        assert_relative_eq!(suit.integrity(), 1.0);
    }

    #[test]
    fn test_repair_capped() {
        let mut suit = PressureSuit::new(SuitTier::Basic);
        suit.repair(0.5);
        assert_relative_eq!(suit.integrity(), 1.0); // Capped at 1.0
    }

    #[test]
    fn test_effective_protection_scales_with_integrity() {
        let mut suit = PressureSuit::new(SuitTier::Basic);
        let full_protection = suit.effective_protection();
        suit.integrity = 0.5;
        let half_protection = suit.effective_protection();
        assert_relative_eq!(half_protection, full_protection * 0.5);
    }

    #[test]
    fn test_apply_protection_within_rating() {
        let suit = PressureSuit::new(SuitTier::Basic);
        let damage = suit.apply_protection(10.0, 100.0);
        assert!(damage < 10.0); // Reduced by suit
        assert!(damage > 0.0); // Not fully blocked
    }

    #[test]
    fn test_apply_protection_beyond_rating() {
        let suit = PressureSuit::new(SuitTier::Basic);
        let damage = suit.apply_protection(10.0, 500.0);
        assert_relative_eq!(damage, 10.0); // No protection beyond rating
    }

    #[test]
    fn test_no_protection_when_unequipped() {
        let mut suit = PressureSuit::new(SuitTier::Basic);
        suit.unequip();
        let damage = suit.apply_protection(10.0, 100.0);
        assert_relative_eq!(damage, 10.0); // No suit = full damage
    }

    use approx::assert_relative_eq;
}
