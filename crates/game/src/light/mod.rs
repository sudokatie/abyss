//! Player light source system for underwater visibility.
//!
//! Implements spec 2.4: Various light sources with different ranges,
//! battery drain rates, and special properties.

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Types of player-carried light sources.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerLight {
    /// Standard headlamp with narrow beam.
    /// 16-block range, low battery drain.
    Headlamp,
    /// Wide-angle floodlight.
    /// 24-block range, high battery drain.
    Floodlight,
    /// Throwable emergency flare.
    /// 8-block range, 30 second duration, no battery.
    Flare,
    /// Passive glow from bioluminescent equipment.
    /// 4-block range, no battery required.
    BioluminescenceGlow,
}

impl PlayerLight {
    /// Get the base range of this light type in blocks.
    #[must_use]
    pub fn base_range(&self) -> f64 {
        match self {
            PlayerLight::Headlamp => 16.0,
            PlayerLight::Floodlight => 24.0,
            PlayerLight::Flare => 8.0,
            PlayerLight::BioluminescenceGlow => 4.0,
        }
    }

    /// Get the battery drain rate per second (0.0 to 1.0 scale, where 1.0 = 100%).
    #[must_use]
    pub fn drain_rate(&self) -> f64 {
        match self {
            PlayerLight::Headlamp => 0.005, // 0.5% per second
            PlayerLight::Floodlight => 0.02, // 2% per second
            PlayerLight::Flare => 0.0,       // No battery, uses duration
            PlayerLight::BioluminescenceGlow => 0.0, // No drain
        }
    }

    /// Check if this light type uses battery.
    #[must_use]
    pub fn uses_battery(&self) -> bool {
        matches!(self, PlayerLight::Headlamp | PlayerLight::Floodlight)
    }

    /// Check if this light type is throwable.
    #[must_use]
    pub fn is_throwable(&self) -> bool {
        matches!(self, PlayerLight::Flare)
    }

    /// Get flare duration in seconds (only applicable to Flare type).
    #[must_use]
    pub fn flare_duration(&self) -> Option<f64> {
        match self {
            PlayerLight::Flare => Some(30.0),
            _ => None,
        }
    }

    /// Get the name of this light type as a string.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            PlayerLight::Headlamp => "Headlamp",
            PlayerLight::Floodlight => "Floodlight",
            PlayerLight::Flare => "Flare",
            PlayerLight::BioluminescenceGlow => "BioluminescenceGlow",
        }
    }
}

/// A single light source carried by a player.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerLightSource {
    /// The type of light.
    pub light_type: PlayerLight,
    /// Whether the light is currently active/on.
    pub active: bool,
    /// Remaining battery as a fraction (0.0 to 1.0).
    /// For flares, this is unused; duration is tracked separately.
    pub battery_remaining: f64,
    /// Current effective range in blocks.
    pub range: f64,
    /// Direction the light was thrown (only for flares).
    pub throw_direction: Option<Vec3>,
    /// Remaining duration for flares in seconds.
    flare_remaining: Option<f64>,
}

impl PlayerLightSource {
    /// Create a new light source of the given type.
    #[must_use]
    pub fn new(light_type: PlayerLight) -> Self {
        Self {
            light_type,
            active: false,
            battery_remaining: 1.0,
            range: light_type.base_range(),
            throw_direction: None,
            flare_remaining: light_type.flare_duration(),
        }
    }

    /// Activate (turn on) this light source.
    pub fn activate(&mut self) {
        if self.can_activate() {
            self.active = true;
        }
    }

    /// Deactivate (turn off) this light source.
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Toggle the light on/off.
    pub fn toggle(&mut self) {
        if self.active {
            self.deactivate();
        } else {
            self.activate();
        }
    }

    /// Check if this light can be activated.
    #[must_use]
    pub fn can_activate(&self) -> bool {
        match self.light_type {
            PlayerLight::Headlamp | PlayerLight::Floodlight => self.battery_remaining > 0.0,
            PlayerLight::Flare => self.flare_remaining.map_or(false, |r| r > 0.0),
            PlayerLight::BioluminescenceGlow => true,
        }
    }

    /// Check if this light is depleted (battery empty or flare expired).
    #[must_use]
    pub fn is_depleted(&self) -> bool {
        match self.light_type {
            PlayerLight::Headlamp | PlayerLight::Floodlight => self.battery_remaining <= 0.0,
            PlayerLight::Flare => self.flare_remaining.map_or(true, |r| r <= 0.0),
            PlayerLight::BioluminescenceGlow => false,
        }
    }

    /// Get remaining flare duration in seconds.
    #[must_use]
    pub fn flare_remaining(&self) -> Option<f64> {
        self.flare_remaining
    }

    /// Throw the flare in a direction (only for Flare type).
    pub fn throw(&mut self, direction: Vec3) {
        if self.light_type.is_throwable() {
            self.throw_direction = Some(direction.normalize_or_zero());
            self.activate();
        }
    }

    /// Update the light source for the given time delta.
    ///
    /// Handles battery drain for battery-powered lights and duration
    /// countdown for flares. Depth can affect drain rate.
    pub fn update(&mut self, delta: f64, _depth: f64) {
        if !self.active {
            return;
        }

        match self.light_type {
            PlayerLight::Headlamp | PlayerLight::Floodlight => {
                let drain = self.light_type.drain_rate() * delta;
                self.battery_remaining = (self.battery_remaining - drain).max(0.0);
                if self.battery_remaining <= 0.0 {
                    self.active = false;
                }
            }
            PlayerLight::Flare => {
                if let Some(remaining) = self.flare_remaining.as_mut() {
                    *remaining = (*remaining - delta).max(0.0);
                    if *remaining <= 0.0 {
                        self.active = false;
                    }
                }
            }
            PlayerLight::BioluminescenceGlow => {
                // No drain, always active if turned on
            }
        }
    }

    /// Recharge the battery by the given amount (0.0 to 1.0 scale).
    pub fn recharge(&mut self, amount: f64) {
        if self.light_type.uses_battery() {
            self.battery_remaining = (self.battery_remaining + amount).min(1.0);
        }
    }
}

/// Inventory of light sources carried by a player.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LightInventory {
    /// All light sources in the inventory.
    lights: Vec<PlayerLightSource>,
    /// Index of the currently active light (if any).
    active_light: Option<usize>,
}

impl LightInventory {
    /// Create a new empty light inventory.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a light source to the inventory.
    pub fn add(&mut self, light: PlayerLightSource) {
        self.lights.push(light);
    }

    /// Remove a light source at the given index.
    pub fn remove(&mut self, index: usize) -> Option<PlayerLightSource> {
        if index >= self.lights.len() {
            return None;
        }

        // Update active_light index if needed
        if let Some(active) = self.active_light {
            if active == index {
                self.active_light = None;
            } else if active > index {
                self.active_light = Some(active - 1);
            }
        }

        Some(self.lights.remove(index))
    }

    /// Get a reference to a light source by index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&PlayerLightSource> {
        self.lights.get(index)
    }

    /// Get a mutable reference to a light source by index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut PlayerLightSource> {
        self.lights.get_mut(index)
    }

    /// Get all light sources.
    #[must_use]
    pub fn lights(&self) -> &[PlayerLightSource] {
        &self.lights
    }

    /// Get the number of light sources.
    #[must_use]
    pub fn len(&self) -> usize {
        self.lights.len()
    }

    /// Check if the inventory is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lights.is_empty()
    }

    /// Get the index of the currently active light.
    #[must_use]
    pub fn active_index(&self) -> Option<usize> {
        self.active_light
    }

    /// Get a reference to the currently active light.
    #[must_use]
    pub fn active(&self) -> Option<&PlayerLightSource> {
        self.active_light.and_then(|i| self.lights.get(i))
    }

    /// Get a mutable reference to the currently active light.
    pub fn active_mut(&mut self) -> Option<&mut PlayerLightSource> {
        self.active_light.and_then(|i| self.lights.get_mut(i))
    }

    /// Activate a light source at the given index.
    ///
    /// Deactivates any currently active light first.
    pub fn activate(&mut self, index: usize) {
        // Deactivate current light
        if let Some(active) = self.active_mut() {
            active.deactivate();
        }

        // Activate new light
        if let Some(light) = self.lights.get_mut(index) {
            light.activate();
            if light.active {
                self.active_light = Some(index);
            }
        }
    }

    /// Deactivate the currently active light.
    pub fn deactivate(&mut self) {
        if let Some(active) = self.active_mut() {
            active.deactivate();
        }
        self.active_light = None;
    }

    /// Update all light sources for the given time delta and depth.
    pub fn update(&mut self, delta: f64, depth: f64) {
        for light in &mut self.lights {
            light.update(delta, depth);
        }

        // Check if active light is now depleted
        if let Some(index) = self.active_light {
            if let Some(light) = self.lights.get(index) {
                if !light.active {
                    self.active_light = None;
                }
            }
        }
    }

    /// Find lights of a specific type.
    #[must_use]
    pub fn find_by_type(&self, light_type: PlayerLight) -> Vec<usize> {
        self.lights
            .iter()
            .enumerate()
            .filter(|(_, l)| l.light_type == light_type)
            .map(|(i, _)| i)
            .collect()
    }

    /// Count lights of a specific type.
    #[must_use]
    pub fn count_by_type(&self, light_type: PlayerLight) -> usize {
        self.lights
            .iter()
            .filter(|l| l.light_type == light_type)
            .count()
    }

    /// Get total light output from all active lights.
    #[must_use]
    pub fn total_light_range(&self) -> f64 {
        self.lights
            .iter()
            .filter(|l| l.active)
            .map(|l| l.range)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headlamp_properties() {
        let lamp = PlayerLight::Headlamp;
        assert_eq!(lamp.base_range(), 16.0);
        assert!((lamp.drain_rate() - 0.005).abs() < f64::EPSILON);
        assert!(lamp.uses_battery());
        assert!(!lamp.is_throwable());
        assert!(lamp.flare_duration().is_none());
    }

    #[test]
    fn test_floodlight_properties() {
        let flood = PlayerLight::Floodlight;
        assert_eq!(flood.base_range(), 24.0);
        assert!((flood.drain_rate() - 0.02).abs() < f64::EPSILON);
        assert!(flood.uses_battery());
    }

    #[test]
    fn test_flare_properties() {
        let flare = PlayerLight::Flare;
        assert_eq!(flare.base_range(), 8.0);
        assert!((flare.drain_rate()).abs() < f64::EPSILON);
        assert!(!flare.uses_battery());
        assert!(flare.is_throwable());
        assert_eq!(flare.flare_duration(), Some(30.0));
    }

    #[test]
    fn test_bioglow_properties() {
        let bio = PlayerLight::BioluminescenceGlow;
        assert_eq!(bio.base_range(), 4.0);
        assert!((bio.drain_rate()).abs() < f64::EPSILON);
        assert!(!bio.uses_battery());
    }

    #[test]
    fn test_light_source_activation() {
        let mut lamp = PlayerLightSource::new(PlayerLight::Headlamp);
        assert!(!lamp.active);
        assert!(lamp.can_activate());

        lamp.activate();
        assert!(lamp.active);

        lamp.deactivate();
        assert!(!lamp.active);

        lamp.toggle();
        assert!(lamp.active);

        lamp.toggle();
        assert!(!lamp.active);
    }

    #[test]
    fn test_headlamp_battery_drain() {
        let mut lamp = PlayerLightSource::new(PlayerLight::Headlamp);
        lamp.activate();
        assert_eq!(lamp.battery_remaining, 1.0);

        // Drain for 10 seconds: 10 * 0.005 = 0.05 (5%)
        lamp.update(10.0, 0.0);
        assert!((lamp.battery_remaining - 0.95).abs() < 0.0001);
        assert!(lamp.active);
    }

    #[test]
    fn test_floodlight_battery_drain() {
        let mut flood = PlayerLightSource::new(PlayerLight::Floodlight);
        flood.activate();

        // Drain for 10 seconds: 10 * 0.02 = 0.2 (20%)
        flood.update(10.0, 0.0);
        assert!((flood.battery_remaining - 0.8).abs() < 0.0001);
    }

    #[test]
    fn test_battery_depletes_turns_off() {
        let mut lamp = PlayerLightSource::new(PlayerLight::Headlamp);
        lamp.battery_remaining = 0.01; // 1% remaining
        lamp.activate();

        // Drain more than remaining
        lamp.update(5.0, 0.0); // Would drain 2.5%
        assert_eq!(lamp.battery_remaining, 0.0);
        assert!(!lamp.active);
        assert!(lamp.is_depleted());
    }

    #[test]
    fn test_flare_expires() {
        let mut flare = PlayerLightSource::new(PlayerLight::Flare);
        assert_eq!(flare.flare_remaining(), Some(30.0));

        flare.activate();
        flare.update(15.0, 0.0);
        assert!((flare.flare_remaining().unwrap() - 15.0).abs() < 0.0001);
        assert!(flare.active);

        flare.update(20.0, 0.0);
        assert_eq!(flare.flare_remaining().unwrap(), 0.0);
        assert!(!flare.active);
        assert!(flare.is_depleted());
    }

    #[test]
    fn test_flare_throw() {
        let mut flare = PlayerLightSource::new(PlayerLight::Flare);
        assert!(flare.throw_direction.is_none());

        flare.throw(Vec3::new(1.0, 0.0, 0.0));
        assert!(flare.active);
        assert!(flare.throw_direction.is_some());
        let dir = flare.throw_direction.unwrap();
        assert!((dir.x - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_bioglow_no_drain() {
        let mut bio = PlayerLightSource::new(PlayerLight::BioluminescenceGlow);
        bio.activate();
        assert!(bio.active);

        bio.update(1000.0, 500.0);
        assert!(bio.active);
        assert!(!bio.is_depleted());
    }

    #[test]
    fn test_recharge_battery() {
        let mut lamp = PlayerLightSource::new(PlayerLight::Headlamp);
        lamp.battery_remaining = 0.5;

        lamp.recharge(0.3);
        assert!((lamp.battery_remaining - 0.8).abs() < 0.0001);

        // Recharge caps at 1.0
        lamp.recharge(0.5);
        assert_eq!(lamp.battery_remaining, 1.0);
    }

    #[test]
    fn test_light_inventory_add_remove() {
        let mut inv = LightInventory::new();
        assert!(inv.is_empty());

        inv.add(PlayerLightSource::new(PlayerLight::Headlamp));
        inv.add(PlayerLightSource::new(PlayerLight::Flare));
        assert_eq!(inv.len(), 2);

        let removed = inv.remove(0);
        assert!(removed.is_some());
        assert_eq!(inv.len(), 1);
    }

    #[test]
    fn test_light_inventory_activate() {
        let mut inv = LightInventory::new();
        inv.add(PlayerLightSource::new(PlayerLight::Headlamp));
        inv.add(PlayerLightSource::new(PlayerLight::Floodlight));

        inv.activate(0);
        assert_eq!(inv.active_index(), Some(0));
        assert!(inv.active().unwrap().active);

        // Activating another deactivates the first
        inv.activate(1);
        assert_eq!(inv.active_index(), Some(1));
        assert!(!inv.get(0).unwrap().active);
        assert!(inv.get(1).unwrap().active);
    }

    #[test]
    fn test_light_inventory_deactivate() {
        let mut inv = LightInventory::new();
        inv.add(PlayerLightSource::new(PlayerLight::Headlamp));
        inv.activate(0);
        assert!(inv.active().is_some());

        inv.deactivate();
        assert!(inv.active().is_none());
        assert!(inv.active_index().is_none());
    }

    #[test]
    fn test_light_inventory_update() {
        let mut inv = LightInventory::new();
        inv.add(PlayerLightSource::new(PlayerLight::Headlamp));
        inv.activate(0);

        inv.update(10.0, 100.0);
        let lamp = inv.get(0).unwrap();
        assert!((lamp.battery_remaining - 0.95).abs() < 0.0001);
    }

    #[test]
    fn test_light_inventory_find_by_type() {
        let mut inv = LightInventory::new();
        inv.add(PlayerLightSource::new(PlayerLight::Headlamp));
        inv.add(PlayerLightSource::new(PlayerLight::Flare));
        inv.add(PlayerLightSource::new(PlayerLight::Headlamp));

        let headlamps = inv.find_by_type(PlayerLight::Headlamp);
        assert_eq!(headlamps.len(), 2);
        assert_eq!(headlamps, vec![0, 2]);

        assert_eq!(inv.count_by_type(PlayerLight::Flare), 1);
    }

    #[test]
    fn test_total_light_range() {
        let mut inv = LightInventory::new();
        assert_eq!(inv.total_light_range(), 0.0);

        inv.add(PlayerLightSource::new(PlayerLight::Headlamp));
        inv.activate(0);
        assert_eq!(inv.total_light_range(), 16.0);
    }

    #[test]
    fn test_depleted_light_cannot_activate() {
        let mut lamp = PlayerLightSource::new(PlayerLight::Headlamp);
        lamp.battery_remaining = 0.0;
        assert!(!lamp.can_activate());

        lamp.activate();
        assert!(!lamp.active);
    }

    #[test]
    fn test_expired_flare_cannot_activate() {
        let mut flare = PlayerLightSource::new(PlayerLight::Flare);
        flare.flare_remaining = Some(0.0);
        assert!(!flare.can_activate());
        assert!(flare.is_depleted());
    }

    #[test]
    fn test_inactive_light_no_drain() {
        let mut lamp = PlayerLightSource::new(PlayerLight::Headlamp);
        // Don't activate
        lamp.update(100.0, 0.0);
        assert_eq!(lamp.battery_remaining, 1.0);
    }

    #[test]
    fn test_remove_active_light_clears_active() {
        let mut inv = LightInventory::new();
        inv.add(PlayerLightSource::new(PlayerLight::Headlamp));
        inv.add(PlayerLightSource::new(PlayerLight::Flare));
        inv.activate(0);

        inv.remove(0);
        assert!(inv.active_index().is_none());
    }
}
