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

use crate::light::{LightInventory, PlayerLight, PlayerLightSource};
use crate::survival::{PlayerDeathManager, RespawnPoint};
use engine_network::protocol::{ClientMessage, ServerMessage};
use glam::Vec3;

use approx::assert_relative_eq;

#[test]
fn test_light_source_battery_drain() {
    let mut inv = LightInventory::new();
    inv.add(PlayerLightSource::new(PlayerLight::Headlamp));
    inv.activate(0);

    let initial_battery = inv.get(0).unwrap().battery_remaining;
    assert_eq!(initial_battery, 1.0);

    // Simulate 100 seconds of use
    inv.update(100.0, 50.0);

    let lamp = inv.get(0).unwrap();
    // 100 seconds * 0.5% per second = 50% drain
    assert!((lamp.battery_remaining - 0.5).abs() < 0.01);
    assert!(lamp.active);
}

#[test]
fn test_flare_expires() {
    let mut flare = PlayerLightSource::new(PlayerLight::Flare);
    flare.activate();
    assert!(flare.active);
    assert_eq!(flare.flare_remaining(), Some(30.0));

    // Burn for 30 seconds
    flare.update(30.0, 100.0);
    assert!(!flare.active);
    assert!(flare.is_depleted());
}

#[test]
fn test_buddy_breath_request() {
    // Verify the BuddyBreathRequest message can be constructed
    let msg = ClientMessage::BuddyBreathRequest { target_id: 12345 };

    // Verify it matches expected structure
    if let ClientMessage::BuddyBreathRequest { target_id } = msg {
        assert_eq!(target_id, 12345);
    } else {
        panic!("Expected BuddyBreathRequest variant");
    }

    // Also verify the server response messages exist
    let oxygen_update = ServerMessage::PlayerOxygenUpdate {
        player_id: 1,
        oxygen: 75.0,
        max_oxygen: 100.0,
    };
    if let ServerMessage::PlayerOxygenUpdate { player_id, oxygen, max_oxygen } = oxygen_update {
        assert_eq!(player_id, 1);
        assert_eq!(oxygen, 75.0);
        assert_eq!(max_oxygen, 100.0);
    }

    let pressure_warning = ServerMessage::PressureWarning {
        player_id: 1,
        depth: 500.0,
        zone_name: "Twilight Zone".to_string(),
    };
    if let ServerMessage::PressureWarning { player_id, depth, zone_name } = pressure_warning {
        assert_eq!(player_id, 1);
        assert_eq!(depth, 500.0);
        assert_eq!(zone_name, "Twilight Zone");
    }

    let light_update = ServerMessage::PlayerLightUpdate {
        player_id: 1,
        light_type: "Headlamp".to_string(),
        active: true,
        range: 16.0,
    };
    if let ServerMessage::PlayerLightUpdate { player_id, light_type, active, range } = light_update {
        assert_eq!(player_id, 1);
        assert_eq!(light_type, "Headlamp");
        assert!(active);
        assert_eq!(range, 16.0);
    }
}

#[test]
fn test_death_and_respawn() {
    let mut manager = PlayerDeathManager::new();
    manager.add_respawn_point(RespawnPoint::new(Vec3::new(0.0, 0.0, 0.0)));

    // Player starts alive
    assert!(manager.is_alive());

    // Player dies at depth
    let death_pos = Vec3::new(100.0, -500.0, 200.0);
    manager.die(death_pos);
    assert!(manager.is_dying());

    // Grace period expires
    manager.update(10.0);
    assert!(manager.is_dead());
    assert_eq!(manager.death_count, 1);

    // Respawn
    let respawn_pos = manager.respawn();
    assert!(respawn_pos.is_some());
    assert!(manager.is_alive());

    // Body can still be recovered after respawn
    assert!(manager.can_recover_body());
}

#[test]
fn test_respawn_at_base() {
    let mut manager = PlayerDeathManager::new();

    // Add a regular respawn point and a base respawn point
    manager.add_respawn_point(RespawnPoint::new(Vec3::new(0.0, 0.0, 0.0)));
    manager.add_respawn_point(RespawnPoint::at_base(
        Vec3::new(500.0, -200.0, 500.0),
        42,
    ));

    // Die and respawn
    manager.die(Vec3::new(100.0, -300.0, 100.0));
    manager.confirm_death();

    // Should prefer the base respawn point
    let best = manager.best_respawn_point().unwrap();
    assert!(best.at_base);
    assert_eq!(best.base_id, Some(42));

    let respawn_pos = manager.respawn().unwrap();
    assert_eq!(respawn_pos.x, 500.0);
    assert_eq!(respawn_pos.y, -200.0);
}
