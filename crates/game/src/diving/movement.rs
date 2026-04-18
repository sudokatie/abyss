//! 3D diving movement with buoyancy and current response.

use glam::Vec3;

/// Base swimming speed (blocks per second).
pub const SWIM_SPEED_BASE: f32 = 4.0;

/// Maximum swim speed with all modifiers.
pub const SWIM_SPEED_MAX: f32 = 12.0;

/// Buoyancy force magnitude.
pub const BUOYANCY_FORCE: f32 = 0.5;

/// Current response factor (how much current pushes player).
pub const CURRENT_RESPONSE: f32 = 0.8;

/// Wall grab deceleration rate.
pub const WALL_GRAB_DECEL: f32 = 10.0;

/// Swimming speed modifier from equipment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwimSpeedModifier {
    /// No modifier.
    None,
    /// Flippers: +50% speed.
    Flippers,
    /// Dive suit: -25% speed (pressure protection trade-off).
    DiveSuit,
    /// Both flippers and dive suit: net +12.5%.
    Both,
}

impl SwimSpeedModifier {
    /// Get the speed multiplier for this modifier.
    #[must_use]
    pub fn multiplier(self) -> f32 {
        match self {
            Self::None => 1.0,
            Self::Flippers => 1.5,
            Self::DiveSuit => 0.75,
            Self::Both => 1.125, // 1.5 * 0.75
        }
    }
}

/// Buoyancy state: affects vertical drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuoyancyState {
    /// Positive buoyancy: drift upward.
    Ascending,
    /// Neutral buoyancy: no vertical drift.
    Neutral,
    /// Negative buoyancy: drift downward.
    Descending,
}

impl Default for BuoyancyState {
    fn default() -> Self {
        Self::Neutral
    }
}

impl BuoyancyState {
    /// Vertical force from buoyancy state.
    #[must_use]
    pub fn vertical_force(self) -> f32 {
        match self {
            Self::Ascending => BUOYANCY_FORCE,
            Self::Neutral => 0.0,
            Self::Descending => -BUOYANCY_FORCE,
        }
    }
}

/// Manages 3D swimming movement for the player.
#[derive(Debug, Clone)]
pub struct DivingMovement {
    /// Current velocity.
    velocity: Vec3,
    /// Whether the player is swimming (underwater).
    swimming: bool,
    /// Whether the player is grabbing a wall.
    wall_grabbed: bool,
    /// Current buoyancy state.
    buoyancy: BuoyancyState,
    /// Speed modifier from equipment.
    speed_modifier: SwimSpeedModifier,
}

impl Default for DivingMovement {
    fn default() -> Self {
        Self::new()
    }
}

impl DivingMovement {
    /// Create a new diving movement controller.
    #[must_use]
    pub fn new() -> Self {
        Self {
            velocity: Vec3::ZERO,
            swimming: false,
            wall_grabbed: false,
            buoyancy: BuoyancyState::Neutral,
            speed_modifier: SwimSpeedModifier::None,
        }
    }

    /// Get current velocity.
    #[must_use]
    pub fn velocity(&self) -> Vec3 {
        self.velocity
    }

    /// Get swim speed after modifiers.
    #[must_use]
    pub fn swim_speed(&self) -> f32 {
        (SWIM_SPEED_BASE * self.speed_modifier.multiplier()).min(SWIM_SPEED_MAX)
    }

    /// Check if swimming.
    #[must_use]
    pub fn is_swimming(&self) -> bool {
        self.swimming
    }

    /// Set swimming state.
    pub fn set_swimming(&mut self, swimming: bool) {
        self.swimming = swimming;
    }

    /// Check if wall is grabbed.
    #[must_use]
    pub fn is_wall_grabbed(&self) -> bool {
        self.wall_grabbed
    }

    /// Grab or release wall.
    pub fn set_wall_grab(&mut self, grabbed: bool) {
        self.wall_grabbed = grabbed;
        if grabbed {
            // Decelerate to stop when grabbing wall
            self.velocity = Vec3::ZERO;
        }
    }

    /// Get buoyancy state.
    #[must_use]
    pub fn buoyancy(&self) -> BuoyancyState {
        self.buoyancy
    }

    /// Set buoyancy state.
    pub fn set_buoyancy(&mut self, state: BuoyancyState) {
        self.buoyancy = state;
    }

    /// Get speed modifier.
    #[must_use]
    pub fn speed_modifier(&self) -> SwimSpeedModifier {
        self.speed_modifier
    }

    /// Set speed modifier.
    pub fn set_speed_modifier(&mut self, modifier: SwimSpeedModifier) {
        self.speed_modifier = modifier;
    }

    /// Apply movement input direction.
    ///
    /// Direction should be normalized. Speed is determined by swim_speed().
    pub fn apply_input(&mut self, direction: Vec3) {
        if !self.swimming {
            return;
        }
        let speed = self.swim_speed();
        self.velocity = direction * speed;
    }

    /// Apply water current force.
    pub fn apply_current(&mut self, current_velocity: Vec3, delta_seconds: f32) {
        if !self.swimming {
            return;
        }
        self.velocity += current_velocity * CURRENT_RESPONSE * delta_seconds;
    }

    /// Apply buoyancy force.
    pub fn apply_buoyancy(&mut self, delta_seconds: f32) {
        if !self.swimming {
            return;
        }
        let force = self.buoyancy.vertical_force();
        self.velocity.y += force * delta_seconds;
    }

    /// Update position based on velocity.
    ///
    /// Returns the position delta for this frame.
    #[must_use]
    pub fn update(&mut self, delta_seconds: f32) -> Vec3 {
        if self.wall_grabbed {
            self.velocity = Vec3::ZERO;
            return Vec3::ZERO;
        }
        let delta = self.velocity * delta_seconds;
        delta
    }

    /// Calculate ascent rate (vertical speed, positive = up).
    #[must_use]
    pub fn ascent_rate(&self) -> f32 {
        self.velocity.y
    }

    /// Calculate depth change rate (positive = deeper).
    #[must_use]
    pub fn descent_rate(&self) -> f32 {
        -self.velocity.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_new_movement() {
        let mov = DivingMovement::new();
        assert_eq!(mov.velocity(), Vec3::ZERO);
        assert!(!mov.is_swimming());
        assert!(!mov.is_wall_grabbed());
        assert_eq!(mov.buoyancy(), BuoyancyState::Neutral);
    }

    #[test]
    fn test_swim_speed_base() {
        let mov = DivingMovement::new();
        assert_relative_eq!(mov.swim_speed(), SWIM_SPEED_BASE);
    }

    #[test]
    fn test_swim_speed_flippers() {
        let mut mov = DivingMovement::new();
        mov.set_speed_modifier(SwimSpeedModifier::Flippers);
        assert_relative_eq!(mov.swim_speed(), SWIM_SPEED_BASE * 1.5);
    }

    #[test]
    fn test_swim_speed_dive_suit() {
        let mut mov = DivingMovement::new();
        mov.set_speed_modifier(SwimSpeedModifier::DiveSuit);
        assert_relative_eq!(mov.swim_speed(), SWIM_SPEED_BASE * 0.75);
    }

    #[test]
    fn test_swim_speed_both() {
        let mut mov = DivingMovement::new();
        mov.set_speed_modifier(SwimSpeedModifier::Both);
        assert_relative_eq!(mov.swim_speed(), SWIM_SPEED_BASE * 1.125);
    }

    #[test]
    fn test_apply_input() {
        let mut mov = DivingMovement::new();
        mov.set_swimming(true);
        mov.apply_input(Vec3::new(1.0, 0.0, 0.0));
        assert_relative_eq!(mov.velocity().x, SWIM_SPEED_BASE);
    }

    #[test]
    fn test_no_input_when_not_swimming() {
        let mut mov = DivingMovement::new();
        mov.apply_input(Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(mov.velocity(), Vec3::ZERO);
    }

    #[test]
    fn test_wall_grab_stops_movement() {
        let mut mov = DivingMovement::new();
        mov.set_swimming(true);
        mov.apply_input(Vec3::new(1.0, 0.0, 0.0));
        mov.set_wall_grab(true);
        assert_eq!(mov.velocity(), Vec3::ZERO);
    }

    #[test]
    fn test_buoyancy_ascending() {
        let mut mov = DivingMovement::new();
        mov.set_swimming(true);
        mov.set_buoyancy(BuoyancyState::Ascending);
        mov.apply_buoyancy(1.0);
        assert!(mov.velocity().y > 0.0);
    }

    #[test]
    fn test_buoyancy_descending() {
        let mut mov = DivingMovement::new();
        mov.set_swimming(true);
        mov.set_buoyancy(BuoyancyState::Descending);
        mov.apply_buoyancy(1.0);
        assert!(mov.velocity().y < 0.0);
    }

    #[test]
    fn test_current_pushes() {
        let mut mov = DivingMovement::new();
        mov.set_swimming(true);
        let current = Vec3::new(2.0, 0.0, 0.0);
        mov.apply_current(current, 1.0);
        assert!(mov.velocity().x > 0.0);
    }

    #[test]
    fn test_update_returns_delta() {
        let mut mov = DivingMovement::new();
        mov.set_swimming(true);
        mov.apply_input(Vec3::new(1.0, 0.0, 0.0));
        let delta = mov.update(0.5);
        assert_relative_eq!(delta.x, SWIM_SPEED_BASE * 0.5);
    }

    #[test]
    fn test_wall_grab_no_delta() {
        let mut mov = DivingMovement::new();
        mov.set_swimming(true);
        mov.set_wall_grab(true);
        let delta = mov.update(1.0);
        assert_eq!(delta, Vec3::ZERO);
    }
}
