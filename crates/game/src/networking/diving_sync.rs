//! Abyss-specific multiplayer synchronization.
//!
//! Handles underwater position sync, oxygen sharing, pressure warnings,
//! and light source visibility for cooperative diving.

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Diver state for network synchronization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiverState {
    /// Player ID.
    pub player_id: u64,
    /// 3D position (x, y=depth, z).
    pub position: Vec3,
    /// Current depth in meters.
    pub depth: f32,
    /// Oxygen level (0-100).
    pub oxygen: f32,
    /// Light source active.
    pub light_on: bool,
    /// Light range in meters.
    pub light_range: f32,
    /// Whether player is incapacitated (can be rescued).
    pub incapacitated: bool,
    /// Whether player is being dragged by another.
    pub being_dragged: bool,
    /// ID of player dragging this one (if any).
    pub dragged_by: Option<u64>,
}

impl DiverState {
    /// Create a new diver state.
    #[must_use]
    pub fn new(player_id: u64) -> Self {
        Self {
            player_id,
            position: Vec3::ZERO,
            depth: 0.0,
            oxygen: 100.0,
            light_on: false,
            light_range: 20.0,
            incapacitated: false,
            being_dragged: false,
            dragged_by: None,
        }
    }

    /// Check if this diver is in a dangerous depth zone.
    #[must_use]
    pub fn pressure_warning(&self) -> PressureWarning {
        if self.depth < 100.0 {
            PressureWarning::Safe
        } else if self.depth < 300.0 {
            PressureWarning::Moderate
        } else if self.depth < 700.0 {
            PressureWarning::High
        } else {
            PressureWarning::Extreme
        }
    }

    /// Check if this diver can share oxygen (buddy breathing).
    #[must_use]
    pub fn can_share_oxygen(&self) -> bool {
        self.oxygen > 20.0 && !self.incapacitated
    }
}

/// Pressure warning level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PressureWarning {
    /// Safe depth (< 100m).
    Safe,
    /// Moderate pressure (100-300m).
    Moderate,
    /// High pressure (300-700m).
    High,
    /// Extreme pressure (> 700m).
    Extreme,
}

impl PressureWarning {
    /// Get the color code for HUD display.
    #[must_use]
    pub fn color_code(&self) -> &str {
        match self {
            PressureWarning::Safe => "green",
            PressureWarning::Moderate => "yellow",
            PressureWarning::High => "red",
            PressureWarning::Extreme => "purple",
        }
    }
}

/// Buddy breathing event.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuddyBreathEvent {
    /// Provider player ID.
    pub provider_id: u64,
    /// Receiver player ID.
    pub receiver_id: u64,
    /// Oxygen amount transferred.
    pub amount: f32,
}

impl BuddyBreathEvent {
    /// Create a new buddy breath event.
    #[must_use]
    pub fn new(provider_id: u64, receiver_id: u64, amount: f32) -> Self {
        Self {
            provider_id,
            receiver_id,
            amount,
        }
    }
}

/// Rescue drag event.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RescueDragEvent {
    /// Rescuer player ID.
    pub rescuer_id: u64,
    /// Victim player ID.
    pub victim_id: u64,
    /// Whether dragging started (true) or stopped (false).
    pub started: bool,
}

impl RescueDragEvent {
    /// Create a new rescue drag event.
    #[must_use]
    pub fn new(rescuer_id: u64, victim_id: u64, started: bool) -> Self {
        Self {
            rescuer_id,
            victim_id,
            started,
        }
    }
}

/// Manages diving sync state for the local client.
#[derive(Clone, Debug, Default)]
pub struct DivingSync {
    /// Known diver states.
    divers: std::collections::HashMap<u64, DiverState>,
    /// Maximum distance for buddy breathing.
    buddy_breath_range: f32,
    /// Maximum distance for rescue drag initiation.
    rescue_range: f32,
}

impl DivingSync {
    /// Create a new diving sync manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            divers: std::collections::HashMap::new(),
            buddy_breath_range: 3.0,
            rescue_range: 5.0,
        }
    }

    /// Register a diver.
    pub fn register_diver(&mut self, state: DiverState) {
        self.divers.insert(state.player_id, state);
    }

    /// Remove a diver.
    pub fn remove_diver(&mut self, player_id: u64) -> bool {
        self.divers.remove(&player_id).is_some()
    }

    /// Update a diver's state.
    pub fn update_diver(&mut self, state: DiverState) {
        self.divers.insert(state.player_id, state);
    }

    /// Get a diver's state.
    #[must_use]
    pub fn get_diver(&self, player_id: u64) -> Option<&DiverState> {
        self.divers.get(&player_id)
    }

    /// Get all divers in buddy breathing range of a position.
    #[must_use]
    pub fn divers_in_buddy_range(&self, pos: Vec3) -> Vec<&DiverState> {
        self.divers
            .values()
            .filter(|d| d.position.distance(pos) <= self.buddy_breath_range)
            .collect()
    }

    /// Get all divers within rescue range of a position.
    #[must_use]
    pub fn divers_in_rescue_range(&self, pos: Vec3) -> Vec<&DiverState> {
        self.divers
            .values()
            .filter(|d| d.position.distance(pos) <= self.rescue_range)
            .collect()
    }

    /// Get all divers at similar depth (party depth check).
    #[must_use]
    pub fn divers_near_depth(&self, depth: f32, tolerance: f32) -> Vec<&DiverState> {
        self.divers
            .values()
            .filter(|d| (d.depth - depth).abs() <= tolerance)
            .collect()
    }

    /// Get all divers needing rescue (incapacitated).
    #[must_use]
    pub fn divers_needing_rescue(&self) -> Vec<&DiverState> {
        self.divers.values().filter(|d| d.incapacitated).collect()
    }

    /// Get divers with active lights visible from a position.
    #[must_use]
    pub fn visible_lights(&self, pos: Vec3) -> Vec<(u64, Vec3, f32)> {
        self.divers
            .values()
            .filter(|d| d.light_on)
            .filter(|d| d.position.distance(pos) <= d.light_range)
            .map(|d| (d.player_id, d.position, d.light_range))
            .collect()
    }

    /// Serialize a diver state for network transmission.
    #[must_use]
    pub fn serialize_diver(state: &DiverState) -> Vec<u8> {
        bincode::serialize(state).unwrap_or_default()
    }

    /// Deserialize a diver state from network data.
    #[must_use]
    pub fn deserialize_diver(data: &[u8]) -> Option<DiverState> {
        bincode::deserialize(data).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diver_state_new() {
        let state = DiverState::new(1);
        assert_eq!(state.player_id, 1);
        assert!((state.depth - 0.0).abs() < f32::EPSILON);
        assert!((state.oxygen - 100.0).abs() < f32::EPSILON);
        assert!(!state.light_on);
        assert!(!state.incapacitated);
    }

    #[test]
    fn test_pressure_warning_safe() {
        let state = DiverState { depth: 50.0, ..DiverState::new(1) };
        assert_eq!(state.pressure_warning(), PressureWarning::Safe);
    }

    #[test]
    fn test_pressure_warning_moderate() {
        let state = DiverState { depth: 200.0, ..DiverState::new(1) };
        assert_eq!(state.pressure_warning(), PressureWarning::Moderate);
    }

    #[test]
    fn test_pressure_warning_high() {
        let state = DiverState { depth: 500.0, ..DiverState::new(1) };
        assert_eq!(state.pressure_warning(), PressureWarning::High);
    }

    #[test]
    fn test_pressure_warning_extreme() {
        let state = DiverState { depth: 800.0, ..DiverState::new(1) };
        assert_eq!(state.pressure_warning(), PressureWarning::Extreme);
    }

    #[test]
    fn test_pressure_warning_colors() {
        assert_eq!(PressureWarning::Safe.color_code(), "green");
        assert_eq!(PressureWarning::Moderate.color_code(), "yellow");
        assert_eq!(PressureWarning::High.color_code(), "red");
        assert_eq!(PressureWarning::Extreme.color_code(), "purple");
    }

    #[test]
    fn test_can_share_oxygen() {
        let state = DiverState { oxygen: 50.0, ..DiverState::new(1) };
        assert!(state.can_share_oxygen());

        let low = DiverState { oxygen: 10.0, ..DiverState::new(1) };
        assert!(!low.can_share_oxygen());

        let incapacitated = DiverState { oxygen: 50.0, incapacitated: true, ..DiverState::new(1) };
        assert!(!incapacitated.can_share_oxygen());
    }

    #[test]
    fn test_buddy_breath_event() {
        let event = BuddyBreathEvent::new(1, 2, 15.0);
        assert_eq!(event.provider_id, 1);
        assert_eq!(event.receiver_id, 2);
        assert!((event.amount - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rescue_drag_event() {
        let event = RescueDragEvent::new(1, 2, true);
        assert_eq!(event.rescuer_id, 1);
        assert_eq!(event.victim_id, 2);
        assert!(event.started);
    }

    #[test]
    fn test_diving_sync_register() {
        let mut sync = DivingSync::new();
        sync.register_diver(DiverState::new(1));
        assert!(sync.get_diver(1).is_some());
    }

    #[test]
    fn test_diving_sync_remove() {
        let mut sync = DivingSync::new();
        sync.register_diver(DiverState::new(1));
        assert!(sync.remove_diver(1));
        assert!(sync.get_diver(1).is_none());
    }

    #[test]
    fn test_diving_sync_buddy_range() {
        let mut sync = DivingSync::new();
        let mut near = DiverState::new(2);
        near.position = Vec3::new(2.0, 0.0, 0.0);
        sync.register_diver(near);

        let mut far = DiverState::new(3);
        far.position = Vec3::new(10.0, 0.0, 0.0);
        sync.register_diver(far);

        let nearby = sync.divers_in_buddy_range(Vec3::ZERO);
        assert_eq!(nearby.len(), 1);
        assert_eq!(nearby[0].player_id, 2);
    }

    #[test]
    fn test_diving_sync_rescue_range() {
        let mut sync = DivingSync::new();
        let mut near = DiverState::new(2);
        near.position = Vec3::new(4.0, 0.0, 0.0);
        near.incapacitated = true;
        sync.register_diver(near);

        let nearby = sync.divers_in_rescue_range(Vec3::ZERO);
        assert_eq!(nearby.len(), 1);
    }

    #[test]
    fn test_diving_sync_near_depth() {
        let mut sync = DivingSync::new();
        let mut shallow = DiverState::new(2);
        shallow.depth = 100.0;
        sync.register_diver(shallow);

        let mut deep = DiverState::new(3);
        deep.depth = 500.0;
        sync.register_diver(deep);

        let nearby = sync.divers_near_depth(110.0, 20.0);
        assert_eq!(nearby.len(), 1);
        assert_eq!(nearby[0].player_id, 2);
    }

    #[test]
    fn test_diving_sync_needing_rescue() {
        let mut sync = DivingSync::new();
        sync.register_diver(DiverState::new(1));

        let mut victim = DiverState::new(2);
        victim.incapacitated = true;
        sync.register_diver(victim);

        assert_eq!(sync.divers_needing_rescue().len(), 1);
    }

    #[test]
    fn test_diving_sync_visible_lights() {
        let mut sync = DivingSync::new();
        sync.register_diver(DiverState::new(1)); // no light

        let mut lit = DiverState::new(2);
        lit.position = Vec3::new(10.0, 0.0, 0.0);
        lit.light_on = true;
        lit.light_range = 20.0;
        sync.register_diver(lit);

        let lights = sync.visible_lights(Vec3::ZERO);
        assert_eq!(lights.len(), 1);
        assert_eq!(lights[0].0, 2);
    }

    #[test]
    fn test_diving_sync_visible_lights_out_of_range() {
        let mut sync = DivingSync::new();
        let mut lit = DiverState::new(2);
        lit.position = Vec3::new(30.0, 0.0, 0.0);
        lit.light_on = true;
        lit.light_range = 20.0;
        sync.register_diver(lit);

        let lights = sync.visible_lights(Vec3::ZERO);
        assert!(lights.is_empty());
    }

    #[test]
    fn test_diving_sync_serialize_deserialize() {
        let mut state = DiverState::new(42);
        state.depth = 250.0;
        state.oxygen = 75.0;
        state.light_on = true;

        let data = DivingSync::serialize_diver(&state);
        let restored = DivingSync::deserialize_diver(&data).unwrap();

        assert_eq!(restored.player_id, 42);
        assert!((restored.depth - 250.0).abs() < f32::EPSILON);
        assert!((restored.oxygen - 75.0).abs() < f32::EPSILON);
        assert!(restored.light_on);
    }
}
