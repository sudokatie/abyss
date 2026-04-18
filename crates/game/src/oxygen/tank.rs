//! Oxygen tank equipment for extended diving.

/// Capacity added per tank.
pub const TANK_CAPACITY: f32 = 50.0;

/// Maximum number of equipped tanks.
pub const MAX_TANKS: usize = 3;

/// Tank equipment slot index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TankSlot(pub usize);

impl TankSlot {
    /// First tank slot.
    pub const SLOT_0: TankSlot = TankSlot(0);
    /// Second tank slot.
    pub const SLOT_1: TankSlot = TankSlot(1);
    /// Third tank slot.
    pub const SLOT_2: TankSlot = TankSlot(2);

    /// All valid slot indices.
    pub const ALL: [TankSlot; 3] = [TankSlot(0), TankSlot(1), TankSlot(2)];
}

/// An oxygen tank that extends player capacity.
#[derive(Debug, Clone)]
pub struct OxygenTank {
    /// Which slot this tank is in.
    slot: TankSlot,
    /// Capacity of this tank.
    capacity: f32,
    /// Whether the tank is equipped.
    equipped: bool,
}

impl OxygenTank {
    /// Create a standard oxygen tank.
    #[must_use]
    pub fn new(slot: TankSlot) -> Self {
        Self {
            slot,
            capacity: TANK_CAPACITY,
            equipped: true,
        }
    }

    /// Create a tank with custom capacity.
    #[must_use]
    pub fn with_capacity(slot: TankSlot, capacity: f32) -> Self {
        Self {
            slot,
            capacity,
            equipped: true,
        }
    }

    /// Get the tank's capacity.
    #[must_use]
    pub fn capacity(&self) -> f32 {
        self.capacity
    }

    /// Get the tank's slot.
    #[must_use]
    pub fn slot(&self) -> TankSlot {
        self.slot
    }

    /// Check if the tank is equipped.
    #[must_use]
    pub fn is_equipped(&self) -> bool {
        self.equipped
    }

    /// Equip the tank.
    pub fn equip(&mut self) {
        self.equipped = true;
    }

    /// Unequip the tank.
    pub fn unequip(&mut self) {
        self.equipped = false;
    }

    /// Get the slot index.
    #[must_use]
    pub fn slot_index(&self) -> usize {
        self.slot.0
    }
}

/// Manages the player's equipped oxygen tanks.
#[derive(Debug, Clone)]
pub struct TankManager {
    /// Equipped tanks (None = empty slot).
    tanks: [Option<OxygenTank>; MAX_TANKS],
}

impl Default for TankManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TankManager {
    /// Create a new tank manager with no tanks equipped.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tanks: [None, None, None],
        }
    }

    /// Equip a tank in the given slot.
    ///
    /// Returns the previously equipped tank if any.
    pub fn equip_tank(&mut self, tank: OxygenTank) -> Option<OxygenTank> {
        let slot_idx = tank.slot_index();
        if slot_idx >= MAX_TANKS {
            return Some(tank); // Invalid slot, return tank
        }
        let old = self.tanks[slot_idx].replace(tank);
        old
    }

    /// Unequip the tank in the given slot.
    pub fn unequip_slot(&mut self, slot: TankSlot) -> Option<OxygenTank> {
        if slot.0 >= MAX_TANKS {
            return None;
        }
        self.tanks[slot.0].take()
    }

    /// Get the tank in the given slot.
    #[must_use]
    pub fn get(&self, slot: TankSlot) -> Option<&OxygenTank> {
        if slot.0 >= MAX_TANKS {
            return None;
        }
        self.tanks[slot.0].as_ref()
    }

    /// Total capacity from all equipped tanks.
    #[must_use]
    pub fn total_capacity(&self) -> f32 {
        self.tanks
            .iter()
            .flatten()
            .filter(|t| t.is_equipped())
            .map(|t| t.capacity())
            .sum()
    }

    /// Number of equipped tanks.
    #[must_use]
    pub fn equipped_count(&self) -> usize {
        self.tanks.iter().flatten().filter(|t| t.is_equipped()).count()
    }

    /// Check if a slot is occupied.
    #[must_use]
    pub fn slot_occupied(&self, slot: TankSlot) -> bool {
        self.tanks.get(slot.0).map_or(false, |t| t.is_some())
    }

    /// Check if all slots are full.
    #[must_use]
    pub fn all_slots_full(&self) -> bool {
        self.tanks.iter().all(|t| t.is_some())
    }

    /// Iterate over equipped tanks.
    pub fn equipped_tanks(&self) -> impl Iterator<Item = &OxygenTank> {
        self.tanks.iter().flatten().filter(|t| t.is_equipped())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tank() {
        let tank = OxygenTank::new(TankSlot::SLOT_0);
        assert_relative_eq!(tank.capacity(), TANK_CAPACITY);
        assert!(tank.is_equipped());
    }

    #[test]
    fn test_custom_capacity() {
        let tank = OxygenTank::with_capacity(TankSlot::SLOT_0, 75.0);
        assert_relative_eq!(tank.capacity(), 75.0);
    }

    #[test]
    fn test_equip_unequip() {
        let mut tank = OxygenTank::new(TankSlot::SLOT_0);
        tank.unequip();
        assert!(!tank.is_equipped());
        tank.equip();
        assert!(tank.is_equipped());
    }

    #[test]
    fn test_tank_manager_new() {
        let mgr = TankManager::new();
        assert_eq!(mgr.equipped_count(), 0);
        assert_relative_eq!(mgr.total_capacity(), 0.0);
    }

    #[test]
    fn test_equip_tank() {
        let mut mgr = TankManager::new();
        mgr.equip_tank(OxygenTank::new(TankSlot::SLOT_0));
        assert_eq!(mgr.equipped_count(), 1);
        assert_relative_eq!(mgr.total_capacity(), TANK_CAPACITY);
    }

    #[test]
    fn test_equip_three_tanks() {
        let mut mgr = TankManager::new();
        mgr.equip_tank(OxygenTank::new(TankSlot::SLOT_0));
        mgr.equip_tank(OxygenTank::new(TankSlot::SLOT_1));
        mgr.equip_tank(OxygenTank::new(TankSlot::SLOT_2));
        assert_eq!(mgr.equipped_count(), 3);
        assert_relative_eq!(mgr.total_capacity(), TANK_CAPACITY * 3.0);
    }

    #[test]
    fn test_unequip_slot() {
        let mut mgr = TankManager::new();
        mgr.equip_tank(OxygenTank::new(TankSlot::SLOT_0));
        let tank = mgr.unequip_slot(TankSlot::SLOT_0);
        assert!(tank.is_some());
        assert_eq!(mgr.equipped_count(), 0);
    }

    #[test]
    fn test_slot_occupied() {
        let mut mgr = TankManager::new();
        assert!(!mgr.slot_occupied(TankSlot::SLOT_0));
        mgr.equip_tank(OxygenTank::new(TankSlot::SLOT_0));
        assert!(mgr.slot_occupied(TankSlot::SLOT_0));
    }

    #[test]
    fn test_all_slots_full() {
        let mut mgr = TankManager::new();
        assert!(!mgr.all_slots_full());
        mgr.equip_tank(OxygenTank::new(TankSlot::SLOT_0));
        mgr.equip_tank(OxygenTank::new(TankSlot::SLOT_1));
        mgr.equip_tank(OxygenTank::new(TankSlot::SLOT_2));
        assert!(mgr.all_slots_full());
    }

    #[test]
    fn test_replace_tank() {
        let mut mgr = TankManager::new();
        mgr.equip_tank(OxygenTank::new(TankSlot::SLOT_0));
        let old = mgr.equip_tank(OxygenTank::with_capacity(TankSlot::SLOT_0, 75.0));
        assert!(old.is_some());
        assert_relative_eq!(mgr.total_capacity(), 75.0);
    }

    #[test]
    fn test_custom_capacity_tank_total() {
        let mut mgr = TankManager::new();
        mgr.equip_tank(OxygenTank::with_capacity(TankSlot::SLOT_0, 100.0));
        mgr.equip_tank(OxygenTank::with_capacity(TankSlot::SLOT_1, 75.0));
        assert_relative_eq!(mgr.total_capacity(), 175.0);
    }

    use approx::assert_relative_eq;
}
