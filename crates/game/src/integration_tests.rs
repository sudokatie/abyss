//! Abyss integration tests.
//!
//! End-to-end tests wiring multiple systems together.

use crate::oxygen::OxygenSupply;
use crate::pressure::{PressureSuit, SuitTier};
use crate::creatures::{HostileCreature, HostileType, PassiveCreature, PassiveType};
use crate::building::{BaseCompartment, Airlock, PowerSystem, PowerGenerator};
use crate::crafting::{DivingEquipment, EquipmentSlot, EquipmentTier, Weapon, WeaponType};
use crate::world::resources::ResourceDistribution;

#[test]
fn test_oxygen_drains_at_depth() {
    let mut oxygen = OxygenSupply::new();
    oxygen.set_underwater(true);
    let surface_level = oxygen.current();
    oxygen.drain(100.0, 10.0); // depth=100m, delta=10s
    assert!(oxygen.current() < surface_level);
}

#[test]
fn test_suit_depth_rating() {
    let suit = PressureSuit::new(SuitTier::Basic);
    assert!(suit.is_within_rating(50.0));
    assert!(!suit.is_within_rating(300.0));
}

#[test]
fn test_combat_at_depth() {
    let mut creature = HostileCreature::new(HostileType::SeaSnake, [5.0, -100.0, 5.0]);
    let mut weapon = Weapon::new(WeaponType::Harpoon);

    assert!(weapon.can_attack());
    weapon.attack();
    creature.take_damage(HostileType::SeaSnake.scaled_damage(100.0));

    creature.take_damage(100.0);
    assert!(!creature.is_alive());
    assert!(!creature.loot().is_empty());
}

#[test]
fn test_base_airlock_cycle() {
    let mut compartment = BaseCompartment::new([0.0, -200.0, 0.0], [10.0, 5.0, 10.0]);
    compartment.has_oxygen_separator = true;
    compartment.has_power = true;

    let mut airlock = Airlock::new([5.0, -200.0, 0.0]);
    assert!(airlock.request_entry());
    airlock.update(6.0);
    assert!(!airlock.flooded);
    assert!(compartment.is_safe());
}

#[test]
fn test_power_system() {
    let mut power = PowerSystem::new(500.0);
    power.add_generator(PowerGenerator::Thermal);
    power.add_generator(PowerGenerator::Bio);
    power.consumption = 8.0;
    power.update(10.0);
    assert!(power.has_power());
}

#[test]
fn test_resource_depth_distribution() {
    let surface = ResourceDistribution::at_depth(50.0);
    let mid = ResourceDistribution::at_depth(400.0);
    let deep = ResourceDistribution::at_depth(1500.0);
    assert!(!surface.available.is_empty());
    assert!(!mid.available.is_empty());
    assert!(!deep.available.is_empty());
}

#[test]
fn test_equipment_benefits() {
    let basic = DivingEquipment::new(EquipmentSlot::Suit, EquipmentTier::Basic);
    let endgame = DivingEquipment::new(EquipmentSlot::Suit, EquipmentTier::Endgame);
    assert!(basic.oxygen_modifier() > endgame.oxygen_modifier());
    assert!(endgame.depth_rating() > basic.depth_rating());
}

#[test]
fn test_passive_creatures() {
    let fish = PassiveCreature::new(PassiveType::FishSchool, [0.0, -50.0, 0.0]);
    let jelly = PassiveCreature::new(PassiveType::Jellyfish, [0.0, -200.0, 0.0]);
    assert!(fish.creature_type.is_catchable());
    assert!(!jelly.creature_type.is_catchable());
    assert!(jelly.creature_type.contact_damage() > 0.0);
}

use approx::assert_relative_eq;
