//! Death consequences and respawn mechanics.
//!
//! Implements spec 6.5.1: on death, drop inventory at death location,
//! respawn at spawn point with full health and hunger.
//!
//! Also implements spec 6.3: DeathState, RespawnPoint, DeathConsequences,
//! and PlayerDeathManager for advanced death/respawn handling.

use engine_core::coords::WorldPos;
use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::inventory::{Inventory, ItemStack};

/// Result of a player death event.
#[derive(Debug, Clone)]
pub struct DeathResult {
    /// Position where the player died (items drop here).
    pub death_position: WorldPos,
    /// Items dropped from inventory.
    pub dropped_items: Vec<ItemStack>,
    /// Damage source that caused the death.
    pub cause: DeathCause,
}

/// Cause of player death.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    /// Fell from a height.
    Fall,
    /// Attacked by an entity.
    Combat,
    /// Drowned underwater.
    Drowning,
    /// Starvation (hunger depleted).
    Starvation,
    /// Burned by fire or lava.
    Fire,
    /// Unknown or generic damage.
    Other,
}

impl DeathCause {
    /// Get a human-readable death message.
    #[must_use]
    pub fn message(&self) -> &'static str {
        match self {
            DeathCause::Fall => "fell to their death",
            DeathCause::Combat => "was slain",
            DeathCause::Drowning => "drowned",
            DeathCause::Starvation => "starved to death",
            DeathCause::Fire => "burned to death",
            DeathCause::Other => "died",
        }
    }
}

/// Manages death state and consequences.
#[derive(Debug, Clone)]
pub struct DeathHandler {
    /// Whether the player is currently dead.
    is_dead: bool,
    /// Time since death in seconds (for death screen timer).
    time_since_death: f32,
    /// Position of last death.
    last_death_pos: Option<WorldPos>,
    /// Cause of last death.
    last_death_cause: Option<DeathCause>,
    /// Whether to keep inventory on death (creative mode, etc.).
    keep_inventory: bool,
}

impl Default for DeathHandler {
    fn default() -> Self {
        Self {
            is_dead: false,
            time_since_death: 0.0,
            last_death_pos: None,
            last_death_cause: None,
            keep_inventory: false,
        }
    }
}

impl DeathHandler {
    /// Create a new death handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle player death.
    ///
    /// Drops all inventory items (unless keep_inventory is set),
    /// records death position and cause.
    pub fn handle_death(
        &mut self,
        position: WorldPos,
        cause: DeathCause,
        inventory: &mut Inventory,
    ) -> DeathResult {
        self.is_dead = true;
        self.time_since_death = 0.0;
        self.last_death_pos = Some(position);
        self.last_death_cause = Some(cause);

        let dropped_items = if self.keep_inventory {
            Vec::new()
        } else {
            Self::drop_inventory(inventory)
        };

        DeathResult {
            death_position: position,
            dropped_items,
            cause,
        }
    }

    /// Drop all inventory items and clear the inventory.
    fn drop_inventory(inventory: &mut Inventory) -> Vec<ItemStack> {
        let mut dropped = Vec::new();

        // Drop all 36 slots
        for slot in 0..36 {
            if let Some(stack) = inventory.remove(slot, u32::MAX) {
                dropped.push(stack);
            }
        }

        dropped
    }

    /// Respawn the player at the given spawn point.
    ///
    /// Clears death state. The caller is responsible for
    /// resetting health and hunger to full.
    pub fn respawn(&mut self) -> Option<DeathCause> {
        let cause = self.last_death_cause;
        self.is_dead = false;
        self.time_since_death = 0.0;
        cause
    }

    /// Check if the player is currently dead.
    #[must_use]
    pub fn is_dead(&self) -> bool {
        self.is_dead
    }

    /// Get time since last death.
    #[must_use]
    pub fn time_since_death(&self) -> f32 {
        self.time_since_death
    }

    /// Get the position of the last death.
    #[must_use]
    pub fn last_death_pos(&self) -> Option<WorldPos> {
        self.last_death_pos
    }

    /// Get the cause of the last death.
    #[must_use]
    pub fn last_death_cause(&self) -> Option<DeathCause> {
        self.last_death_cause
    }

    /// Update death timer (call each frame while dead).
    pub fn tick(&mut self, dt: f32) {
        if self.is_dead {
            self.time_since_death += dt;
        }
    }

    /// Set keep inventory mode.
    pub fn set_keep_inventory(&mut self, keep: bool) {
        self.keep_inventory = keep;
    }

    /// Check if keep inventory is enabled.
    #[must_use]
    pub fn keeps_inventory(&self) -> bool {
        self.keep_inventory
    }
}

/// Dropped item entity in the world.
///
/// Items dropped on death float at the death position
/// and can be picked up by any player.
#[derive(Debug, Clone)]
pub struct DroppedItem {
    /// The item stack that was dropped.
    pub stack: ItemStack,
    /// World position where the item was dropped.
    pub position: WorldPos,
    /// Time the item has been on the ground (for despawn timer).
    pub age_secs: f32,
    /// Whether the item can be picked up yet (brief pickup delay).
    pub can_pickup: bool,
}

/// Despawn time for dropped items (5 minutes).
pub const ITEM_DESPAWN_TIME: f32 = 300.0;

/// Pickup delay after dropping (prevents instant re-pickup).
pub const PICKUP_DELAY_SECS: f32 = 0.5;

impl DroppedItem {
    /// Create a new dropped item.
    #[must_use]
    pub fn new(stack: ItemStack, position: WorldPos) -> Self {
        Self {
            stack,
            position,
            age_secs: 0.0,
            can_pickup: false,
        }
    }

    /// Update the dropped item each frame.
    pub fn tick(&mut self, dt: f32) {
        self.age_secs += dt;
        if self.age_secs >= PICKUP_DELAY_SECS {
            self.can_pickup = true;
        }
    }

    /// Check if this item should be despawned.
    #[must_use]
    pub fn should_despawn(&self) -> bool {
        self.age_secs >= ITEM_DESPAWN_TIME
    }
}

// ============================================================================
// Spec 6.3: Advanced Death and Respawn System
// ============================================================================

/// Current state of a player's life/death cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeathState {
    /// Player is alive and active.
    Alive,
    /// Player is dying (grace period for potential rescue).
    Dying,
    /// Player is dead, waiting for respawn.
    Dead,
}

impl Default for DeathState {
    fn default() -> Self {
        DeathState::Alive
    }
}

/// A location where a player can respawn.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RespawnPoint {
    /// World position of the respawn point.
    pub position: Vec3,
    /// Whether this respawn point is at an underwater base.
    pub at_base: bool,
    /// ID of the base (if at a base).
    pub base_id: Option<u64>,
}

impl RespawnPoint {
    /// Create a new respawn point at a position.
    #[must_use]
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            at_base: false,
            base_id: None,
        }
    }

    /// Create a respawn point at a base.
    #[must_use]
    pub fn at_base(position: Vec3, base_id: u64) -> Self {
        Self {
            position,
            at_base: true,
            base_id: Some(base_id),
        }
    }
}

/// Consequences of player death.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeathConsequences {
    /// Whether unequipped items are lost on death.
    pub lose_unequipped: bool,
    /// Position of the player's body (for item recovery).
    pub body_position: Option<Vec3>,
    /// Timer for body recovery window in seconds.
    pub recovery_timer: f64,
}

impl Default for DeathConsequences {
    fn default() -> Self {
        Self {
            lose_unequipped: true,
            body_position: None,
            recovery_timer: 0.0,
        }
    }
}

impl DeathConsequences {
    /// Create new death consequences.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if body recovery is still available.
    #[must_use]
    pub fn can_recover(&self) -> bool {
        self.body_position.is_some() && self.recovery_timer > 0.0
    }
}

/// Default grace period for dying state (seconds).
pub const DYING_GRACE_PERIOD: f64 = 5.0;

/// Default body recovery window (seconds).
pub const BODY_RECOVERY_WINDOW: f64 = 300.0;

/// Manager for player death state and respawning.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerDeathManager {
    /// Current death state.
    pub state: DeathState,
    /// Available respawn points.
    respawn_points: Vec<RespawnPoint>,
    /// Current death consequences.
    pub consequences: DeathConsequences,
    /// Total number of deaths.
    pub death_count: u32,
    /// Timer for dying grace period.
    dying_timer: f64,
}

impl Default for PlayerDeathManager {
    fn default() -> Self {
        Self {
            state: DeathState::Alive,
            respawn_points: Vec::new(),
            consequences: DeathConsequences::default(),
            death_count: 0,
            dying_timer: 0.0,
        }
    }
}

impl PlayerDeathManager {
    /// Create a new death manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Initiate the dying process.
    ///
    /// Places the player in the Dying state with a grace period.
    /// If already dying or dead, this has no effect.
    pub fn die(&mut self, body_position: Vec3) {
        if self.state != DeathState::Alive {
            return;
        }

        self.state = DeathState::Dying;
        self.dying_timer = DYING_GRACE_PERIOD;
        self.consequences.body_position = Some(body_position);
        self.consequences.recovery_timer = BODY_RECOVERY_WINDOW;
    }

    /// Transition from Dying to Dead state immediately.
    pub fn confirm_death(&mut self) {
        if self.state == DeathState::Dying {
            self.state = DeathState::Dead;
            self.death_count += 1;
            self.dying_timer = 0.0;
        }
    }

    /// Respawn the player at the best available respawn point.
    ///
    /// Returns the respawn position, or None if no respawn points are available.
    pub fn respawn(&mut self) -> Option<Vec3> {
        if self.state != DeathState::Dead {
            return None;
        }

        let respawn_pos = self.best_respawn_point().map(|rp| rp.position);

        if respawn_pos.is_some() {
            self.state = DeathState::Alive;
            self.dying_timer = 0.0;
            // Note: body recovery timer continues even after respawn
        }

        respawn_pos
    }

    /// Add a respawn point.
    pub fn add_respawn_point(&mut self, point: RespawnPoint) {
        self.respawn_points.push(point);
    }

    /// Remove a respawn point by base ID.
    pub fn remove_respawn_point(&mut self, base_id: u64) {
        self.respawn_points.retain(|p| p.base_id != Some(base_id));
    }

    /// Get all respawn points.
    #[must_use]
    pub fn respawn_points(&self) -> &[RespawnPoint] {
        &self.respawn_points
    }

    /// Get the best respawn point (prefers base respawns).
    #[must_use]
    pub fn best_respawn_point(&self) -> Option<&RespawnPoint> {
        // Prefer base respawn points, then any respawn point
        self.respawn_points
            .iter()
            .find(|p| p.at_base)
            .or_else(|| self.respawn_points.first())
    }

    /// Check if body can still be recovered.
    #[must_use]
    pub fn can_recover_body(&self) -> bool {
        self.consequences.can_recover()
    }

    /// Attempt to recover the body (retrieve dropped items).
    ///
    /// Returns the body position if recovery is successful.
    pub fn recover_body(&mut self) -> Option<Vec3> {
        if !self.can_recover_body() {
            return None;
        }

        let pos = self.consequences.body_position.take();
        self.consequences.recovery_timer = 0.0;
        pos
    }

    /// Update the death manager each frame.
    pub fn update(&mut self, delta: f64) {
        // Update dying timer
        if self.state == DeathState::Dying {
            self.dying_timer -= delta;
            if self.dying_timer <= 0.0 {
                self.confirm_death();
            }
        }

        // Update body recovery timer
        if self.consequences.recovery_timer > 0.0 {
            self.consequences.recovery_timer -= delta;
            if self.consequences.recovery_timer <= 0.0 {
                self.consequences.body_position = None;
            }
        }
    }

    /// Get remaining dying grace period.
    #[must_use]
    pub fn dying_timer(&self) -> f64 {
        self.dying_timer
    }

    /// Check if the player is alive.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.state == DeathState::Alive
    }

    /// Check if the player is in the dying state.
    #[must_use]
    pub fn is_dying(&self) -> bool {
        self.state == DeathState::Dying
    }

    /// Check if the player is dead.
    #[must_use]
    pub fn is_dead(&self) -> bool {
        self.state == DeathState::Dead
    }

    /// Rescue the player from dying state (buddy saves them).
    ///
    /// Returns true if rescue was successful.
    pub fn rescue(&mut self) -> bool {
        if self.state == DeathState::Dying {
            self.state = DeathState::Alive;
            self.dying_timer = 0.0;
            self.consequences.body_position = None;
            self.consequences.recovery_timer = 0.0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::ItemId;

    fn make_stack(id: u16, count: u32) -> ItemStack {
        ItemStack::new(ItemId(id), count)
    }

    fn add_to_inventory(inv: &mut Inventory, stack: ItemStack) {
        let leftover = inv.add(stack);
        assert!(leftover.is_none(), "inventory should have room");
    }

    #[test]
    fn test_death_drops_inventory() {
        let mut handler = DeathHandler::new();
        let mut inventory = Inventory::new();

        add_to_inventory(&mut inventory, make_stack(1, 64));
        add_to_inventory(&mut inventory, make_stack(2, 32));

        let result = handler.handle_death(
            WorldPos::new(10, 20, 30),
            DeathCause::Fall,
            &mut inventory,
        );

        assert!(handler.is_dead());
        assert_eq!(result.dropped_items.len(), 2);
        assert_eq!(result.death_position, WorldPos::new(10, 20, 30));
        assert_eq!(result.cause, DeathCause::Fall);
    }

    #[test]
    fn test_keep_inventory_no_drops() {
        let mut handler = DeathHandler::new();
        handler.set_keep_inventory(true);

        let mut inventory = Inventory::new();
        add_to_inventory(&mut inventory, make_stack(1, 64));

        let result = handler.handle_death(
            WorldPos::new(0, 0, 0),
            DeathCause::Combat,
            &mut inventory,
        );

        assert!(result.dropped_items.is_empty());
        assert!(handler.keeps_inventory());
    }

    #[test]
    fn test_respawn_clears_death() {
        let mut handler = DeathHandler::new();
        let mut inventory = Inventory::new();

        handler.handle_death(
            WorldPos::new(5, 10, 15),
            DeathCause::Drowning,
            &mut inventory,
        );

        assert!(handler.is_dead());

        let cause = handler.respawn();
        assert!(!handler.is_dead());
        assert_eq!(cause, Some(DeathCause::Drowning));
        assert_eq!(handler.time_since_death(), 0.0);
    }

    #[test]
    fn test_death_cause_messages() {
        assert_eq!(DeathCause::Fall.message(), "fell to their death");
        assert_eq!(DeathCause::Combat.message(), "was slain");
        assert_eq!(DeathCause::Drowning.message(), "drowned");
        assert_eq!(DeathCause::Starvation.message(), "starved to death");
        assert_eq!(DeathCause::Fire.message(), "burned to death");
        assert_eq!(DeathCause::Other.message(), "died");
    }

    #[test]
    fn test_death_timer() {
        let mut handler = DeathHandler::new();
        let mut inventory = Inventory::new();

        handler.handle_death(
            WorldPos::new(0, 0, 0),
            DeathCause::Other,
            &mut inventory,
        );

        handler.tick(1.0);
        assert!((handler.time_since_death() - 1.0).abs() < 0.001);

        handler.tick(0.5);
        assert!((handler.time_since_death() - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_timer_stops_on_respawn() {
        let mut handler = DeathHandler::new();
        let mut inventory = Inventory::new();

        handler.handle_death(
            WorldPos::new(0, 0, 0),
            DeathCause::Other,
            &mut inventory,
        );

        handler.tick(2.0);
        handler.respawn();
        handler.tick(1.0); // Should not increment after respawn

        assert_eq!(handler.time_since_death(), 0.0);
    }

    #[test]
    fn test_dropped_item_pickup_delay() {
        let item = DroppedItem::new(make_stack(1, 10), WorldPos::new(0, 0, 0));
        assert!(!item.can_pickup);
        assert!(!item.should_despawn());
    }

    #[test]
    fn test_dropped_item_becomes_pickupable() {
        let mut item = DroppedItem::new(make_stack(1, 10), WorldPos::new(0, 0, 0));
        item.tick(0.5);
        assert!(item.can_pickup);
    }

    #[test]
    fn test_dropped_item_despawn() {
        let mut item = DroppedItem::new(make_stack(1, 10), WorldPos::new(0, 0, 0));
        item.tick(300.0);
        assert!(item.should_despawn());
    }

    #[test]
    fn test_last_death_tracking() {
        let mut handler = DeathHandler::new();
        assert_eq!(handler.last_death_pos(), None);
        assert_eq!(handler.last_death_cause(), None);

        let mut inventory = Inventory::new();
        handler.handle_death(
            WorldPos::new(42, 64, 100),
            DeathCause::Fire,
            &mut inventory,
        );

        assert_eq!(handler.last_death_pos(), Some(WorldPos::new(42, 64, 100)));
        assert_eq!(handler.last_death_cause(), Some(DeathCause::Fire));
    }

    // ========================================================================
    // Tests for Spec 6.3: PlayerDeathManager
    // ========================================================================

    #[test]
    fn test_death_state_transitions() {
        let mut manager = PlayerDeathManager::new();
        assert!(manager.is_alive());
        assert_eq!(manager.state, DeathState::Alive);

        manager.die(Vec3::new(10.0, -100.0, 20.0));
        assert!(manager.is_dying());
        assert_eq!(manager.state, DeathState::Dying);

        manager.confirm_death();
        assert!(manager.is_dead());
        assert_eq!(manager.state, DeathState::Dead);
        assert_eq!(manager.death_count, 1);
    }

    #[test]
    fn test_dying_grace_period() {
        let mut manager = PlayerDeathManager::new();
        manager.die(Vec3::new(0.0, 0.0, 0.0));

        assert!(manager.is_dying());
        assert!((manager.dying_timer() - DYING_GRACE_PERIOD).abs() < 0.001);

        // Update but not enough to expire
        manager.update(2.0);
        assert!(manager.is_dying());

        // Update to expire grace period
        manager.update(4.0);
        assert!(manager.is_dead());
    }

    #[test]
    fn test_respawn_points() {
        let mut manager = PlayerDeathManager::new();

        manager.add_respawn_point(RespawnPoint::new(Vec3::new(0.0, 0.0, 0.0)));
        manager.add_respawn_point(RespawnPoint::at_base(Vec3::new(100.0, -50.0, 100.0), 42));

        assert_eq!(manager.respawn_points().len(), 2);

        // Best respawn should prefer base
        let best = manager.best_respawn_point().unwrap();
        assert!(best.at_base);
        assert_eq!(best.base_id, Some(42));
    }

    #[test]
    fn test_respawn_restores_alive() {
        let mut manager = PlayerDeathManager::new();
        manager.add_respawn_point(RespawnPoint::new(Vec3::new(50.0, 0.0, 50.0)));

        manager.die(Vec3::new(0.0, 0.0, 0.0));
        manager.confirm_death();
        assert!(manager.is_dead());

        let respawn_pos = manager.respawn();
        assert!(respawn_pos.is_some());
        assert!(manager.is_alive());
    }

    #[test]
    fn test_body_recovery() {
        let mut manager = PlayerDeathManager::new();
        manager.add_respawn_point(RespawnPoint::new(Vec3::ZERO));

        let body_pos = Vec3::new(10.0, -200.0, 30.0);
        manager.die(body_pos);
        manager.confirm_death();
        manager.respawn();

        assert!(manager.can_recover_body());
        let recovered_pos = manager.recover_body();
        assert_eq!(recovered_pos, Some(body_pos));
        assert!(!manager.can_recover_body());
    }

    #[test]
    fn test_body_recovery_expires() {
        let mut manager = PlayerDeathManager::new();
        manager.add_respawn_point(RespawnPoint::new(Vec3::ZERO));

        manager.die(Vec3::new(0.0, 0.0, 0.0));
        manager.confirm_death();
        manager.respawn();

        // Update past recovery window
        manager.update(BODY_RECOVERY_WINDOW + 1.0);
        assert!(!manager.can_recover_body());
        assert!(manager.recover_body().is_none());
    }

    #[test]
    fn test_rescue_from_dying() {
        let mut manager = PlayerDeathManager::new();
        manager.die(Vec3::new(0.0, 0.0, 0.0));
        assert!(manager.is_dying());

        let rescued = manager.rescue();
        assert!(rescued);
        assert!(manager.is_alive());
        assert_eq!(manager.death_count, 0);
    }

    #[test]
    fn test_cannot_rescue_when_dead() {
        let mut manager = PlayerDeathManager::new();
        manager.die(Vec3::new(0.0, 0.0, 0.0));
        manager.confirm_death();

        let rescued = manager.rescue();
        assert!(!rescued);
        assert!(manager.is_dead());
    }

    #[test]
    fn test_remove_respawn_point() {
        let mut manager = PlayerDeathManager::new();
        manager.add_respawn_point(RespawnPoint::at_base(Vec3::ZERO, 1));
        manager.add_respawn_point(RespawnPoint::at_base(Vec3::ONE, 2));
        assert_eq!(manager.respawn_points().len(), 2);

        manager.remove_respawn_point(1);
        assert_eq!(manager.respawn_points().len(), 1);
        assert_eq!(manager.respawn_points()[0].base_id, Some(2));
    }

    #[test]
    fn test_no_respawn_without_points() {
        let mut manager = PlayerDeathManager::new();
        manager.die(Vec3::ZERO);
        manager.confirm_death();

        let pos = manager.respawn();
        assert!(pos.is_none());
        assert!(manager.is_dead()); // Still dead, no respawn point
    }
}
