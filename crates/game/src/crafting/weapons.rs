//! Underwater weapons and tools.
//!
//! Harpoon, depth charge, electric prod, artifact weapons.
//! Each has damage, range, and special effects.

/// Weapon type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponType {
    /// Ranged, retrievable spear. 6 damage, 16 block range.
    Harpoon,
    /// Explosive. 20 damage radius, breaks blocks, dangerous to user.
    DepthCharge,
    /// Melee. 4 damage + stun, powered by jellyfish cells.
    ElectricProd,
    /// Endgame harpoon. 15 damage, 24 range, returns instantly.
    ArtifactHarpoon,
    /// Endgame blade. 12 damage + bleed, ignores armor.
    AbyssalBlade,
}

impl WeaponType {
    /// Human-readable name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            WeaponType::Harpoon => "Harpoon",
            WeaponType::DepthCharge => "Depth Charge",
            WeaponType::ElectricProd => "Electric Prod",
            WeaponType::ArtifactHarpoon => "Artifact Harpoon",
            WeaponType::AbyssalBlade => "Abyssal Blade",
        }
    }

    /// Base damage per hit.
    #[must_use]
    pub fn damage(&self) -> f32 {
        match self {
            WeaponType::Harpoon => 6.0,
            WeaponType::DepthCharge => 20.0,
            WeaponType::ElectricProd => 4.0,
            WeaponType::ArtifactHarpoon => 15.0,
            WeaponType::AbyssalBlade => 12.0,
        }
    }

    /// Attack range (blocks).
    #[must_use]
    pub fn range(&self) -> f32 {
        match self {
            WeaponType::Harpoon => 16.0,
            WeaponType::DepthCharge => 8.0, // Throw range
            WeaponType::ElectricProd => 3.0,
            WeaponType::ArtifactHarpoon => 24.0,
            WeaponType::AbyssalBlade => 4.0,
        }
    }

    /// Attack cooldown (seconds).
    #[must_use]
    pub fn cooldown(&self) -> f32 {
        match self {
            WeaponType::Harpoon => 1.5,
            WeaponType::DepthCharge => 3.0,
            WeaponType::ElectricProd => 0.8,
            WeaponType::ArtifactHarpoon => 1.0,
            WeaponType::AbyssalBlade => 0.6,
        }
    }

    /// Whether this weapon is ranged.
    #[must_use]
    pub fn is_ranged(&self) -> bool {
        match self {
            WeaponType::Harpoon | WeaponType::DepthCharge | WeaponType::ArtifactHarpoon => true,
            _ => false,
        }
    }

    /// Whether this is an endgame weapon.
    #[must_use]
    pub fn is_endgame(&self) -> bool {
        matches!(self, WeaponType::ArtifactHarpoon | WeaponType::AbyssalBlade)
    }

    /// Blast radius (0 for non-AOE).
    #[must_use]
    pub fn blast_radius(&self) -> f32 {
        match self {
            WeaponType::DepthCharge => 5.0,
            _ => 0.0,
        }
    }

    /// Self-damage at close range (depth charges are dangerous).
    #[must_use]
    pub fn self_damage(&self, distance: f32) -> f32 {
        match self {
            WeaponType::DepthCharge => {
                if distance < self.blast_radius() {
                    self.damage() * (1.0 - distance / self.blast_radius())
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    /// Stun duration (0 for no stun).
    #[must_use]
    pub fn stun_duration(&self) -> f32 {
        match self {
            WeaponType::ElectricProd => 2.0,
            _ => 0.0,
        }
    }

    /// Whether weapon returns to user after thrown.
    #[must_use]
    pub fn returns_to_user(&self) -> bool {
        matches!(self, WeaponType::Harpoon | WeaponType::ArtifactHarpoon)
    }

    /// Return time (seconds).
    #[must_use]
    pub fn return_time(&self) -> f32 {
        match self {
            WeaponType::Harpoon => 1.0,
            WeaponType::ArtifactHarpoon => 0.3, // Instant return
            _ => 0.0,
        }
    }
}

/// A weapon instance with durability.
#[derive(Debug, Clone)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub durability: f32,
    pub max_durability: f32,
    pub cooldown_remaining: f32,
}

impl Weapon {
    /// Create a new weapon.
    #[must_use]
    pub fn new(weapon_type: WeaponType) -> Self {
        let max_durability = match weapon_type {
            WeaponType::Harpoon => 200.0,
            WeaponType::DepthCharge => 1.0, // Single use
            WeaponType::ElectricProd => 150.0,
            WeaponType::ArtifactHarpoon => f32::MAX, // Indestructible
            WeaponType::AbyssalBlade => f32::MAX,
        };
        Self {
            weapon_type,
            durability: max_durability,
            max_durability,
            cooldown_remaining: 0.0,
        }
    }

    /// Check if weapon can attack.
    #[must_use]
    pub fn can_attack(&self) -> bool {
        self.cooldown_remaining <= 0.0 && self.durability > 0.0
    }

    /// Start an attack, setting cooldown.
    pub fn attack(&mut self) -> bool {
        if !self.can_attack() {
            return false;
        }
        self.cooldown_remaining = self.weapon_type.cooldown();
        // Depth charges are single use
        if self.weapon_type == WeaponType::DepthCharge {
            self.durability = 0.0;
        }
        true
    }

    /// Update cooldown.
    pub fn update(&mut self, delta: f32) {
        self.cooldown_remaining = (self.cooldown_remaining - delta).max(0.0);
    }

    /// Check if weapon is broken.
    #[must_use]
    pub fn is_broken(&self) -> bool {
        self.durability <= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harpoon_stats() {
        assert_relative_eq!(WeaponType::Harpoon.damage(), 6.0);
        assert_relative_eq!(WeaponType::Harpoon.range(), 16.0);
        assert!(WeaponType::Harpoon.returns_to_user());
    }

    #[test]
    fn test_depth_charge_aoe() {
        assert!(WeaponType::DepthCharge.blast_radius() > 0.0);
        assert!(WeaponType::DepthCharge.self_damage(1.0) > 0.0);
        assert_relative_eq!(WeaponType::DepthCharge.self_damage(10.0), 0.0);
    }

    #[test]
    fn test_electric_prod_stun() {
        assert!(WeaponType::ElectricProd.stun_duration() > 0.0);
    }

    #[test]
    fn test_weapon_attack_cooldown() {
        let mut weapon = Weapon::new(WeaponType::Harpoon);
        assert!(weapon.can_attack());
        weapon.attack();
        assert!(!weapon.can_attack());
        weapon.update(2.0);
        assert!(weapon.can_attack());
    }

    #[test]
    fn test_depth_charge_single_use() {
        let mut weapon = Weapon::new(WeaponType::DepthCharge);
        weapon.attack();
        assert!(weapon.is_broken());
    }

    #[test]
    fn test_artifact_indestructible() {
        let weapon = Weapon::new(WeaponType::ArtifactHarpoon);
        assert!(!weapon.is_broken());
    }

    #[test]
    fn test_endgame_weapons() {
        assert!(WeaponType::ArtifactHarpoon.is_endgame());
        assert!(WeaponType::AbyssalBlade.is_endgame());
        assert!(!WeaponType::Harpoon.is_endgame());
    }

    use approx::assert_relative_eq;
}
