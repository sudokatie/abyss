//! Swimming AI for underwater creatures.
//!
//! 3D movement patterns: patrol, chase, flee.
//! Depth range constraints and current response.

/// Default patrol radius (blocks).
pub const PATROL_RADIUS: f32 = 20.0;

/// Default chase speed (blocks/sec).
pub const CHASE_SPEED: f32 = 6.0;

/// Default flee speed (blocks/sec).
pub const FLEE_SPEED: f32 = 8.0;

/// Default patrol speed (blocks/sec).
pub const PATROL_SPEED: f32 = 2.0;

/// Detection range for chasing (blocks).
pub const DETECTION_RANGE: f32 = 30.0;

/// Flee threshold (health percentage).
pub const FLEE_HEALTH_THRESHOLD: f32 = 0.3;

/// Swimming AI state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwimAIState {
    /// Patrolling within home area.
    Patrol,
    /// Chasing a detected target.
    Chase,
    /// Fleeing from danger.
    Flee,
    /// Returning to depth range after straying.
    Returning,
}

/// Configuration for a swimming creature's AI.
#[derive(Debug, Clone)]
pub struct SwimAIConfig {
    /// Minimum depth this creature can operate at.
    pub min_depth: f32,
    /// Maximum depth this creature can operate at.
    pub max_depth: f32,
    /// Swim speed (blocks/sec).
    pub swim_speed: f32,
    /// Chase speed multiplier.
    pub chase_speed: f32,
    /// Flee speed multiplier.
    pub flee_speed: f32,
    /// Detection range for chasing.
    pub detection_range: f32,
    /// Health threshold to start fleeing.
    pub flee_health_threshold: f32,
    /// Patrol center position.
    pub home_position: [f32; 3],
    /// Patrol radius.
    pub patrol_radius: f32,
}

impl SwimAIConfig {
    /// Create config for a shallow creature (0-200m).
    #[must_use]
    pub fn shallow(home: [f32; 3]) -> Self {
        Self {
            min_depth: 0.0,
            max_depth: 200.0,
            swim_speed: PATROL_SPEED,
            chase_speed: CHASE_SPEED,
            flee_speed: FLEE_SPEED,
            detection_range: DETECTION_RANGE,
            flee_health_threshold: FLEE_HEALTH_THRESHOLD,
            home_position: home,
            patrol_radius: PATROL_RADIUS,
        }
    }

    /// Create config for a mid-depth creature (200-500m).
    #[must_use]
    pub fn mid_depth(home: [f32; 3]) -> Self {
        Self {
            min_depth: 200.0,
            max_depth: 500.0,
            swim_speed: PATROL_SPEED * 0.9,
            chase_speed: CHASE_SPEED * 0.8,
            flee_speed: FLEE_SPEED * 0.9,
            detection_range: DETECTION_RANGE * 0.8,
            flee_health_threshold: FLEE_HEALTH_THRESHOLD,
            home_position: home,
            patrol_radius: PATROL_RADIUS * 1.2,
        }
    }

    /// Create config for a deep creature (500m+).
    #[must_use]
    pub fn deep(home: [f32; 3]) -> Self {
        Self {
            min_depth: 500.0,
            max_depth: 2000.0,
            swim_speed: PATROL_SPEED * 0.7,
            chase_speed: CHASE_SPEED * 0.6,
            flee_speed: FLEE_SPEED * 0.7,
            detection_range: DETECTION_RANGE * 0.6,
            flee_health_threshold: FLEE_HEALTH_THRESHOLD,
            home_position: home,
            patrol_radius: PATROL_RADIUS * 1.5,
        }
    }

    /// Check if a depth is within this creature's range.
    #[must_use]
    pub fn is_in_depth_range(&self, depth: f32) -> bool {
        depth >= self.min_depth && depth <= self.max_depth
    }
}

/// Swimming AI controller.
#[derive(Debug, Clone)]
pub struct SwimAI {
    config: SwimAIConfig,
    state: SwimAIState,
    position: [f32; 3],
    velocity: [f32; 3],
    patrol_angle: f32,
    health_fraction: f32,
    target_position: Option<[f32; 3]>,
}

impl SwimAI {
    /// Create a new swimming AI.
    #[must_use]
    pub fn new(config: SwimAIConfig, start_position: [f32; 3]) -> Self {
        Self {
            config,
            state: SwimAIState::Patrol,
            position: start_position,
            velocity: [0.0, 0.0, 0.0],
            patrol_angle: 0.0,
            health_fraction: 1.0,
            target_position: None,
        }
    }

    /// Get current AI state.
    #[must_use]
    pub fn state(&self) -> SwimAIState {
        self.state
    }

    /// Get current position.
    #[must_use]
    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    /// Get current velocity.
    #[must_use]
    pub fn velocity(&self) -> [f32; 3] {
        self.velocity
    }

    /// Set health fraction (0.0 to 1.0).
    pub fn set_health(&mut self, fraction: f32) {
        self.health_fraction = fraction.clamp(0.0, 1.0);
    }

    /// Set a target position (player position for chase/flee).
    pub fn set_target(&mut self, pos: Option<[f32; 3]>) {
        self.target_position = pos;
    }

    /// Distance to target.
    #[must_use]
    pub fn distance_to_target(&self) -> Option<f32> {
        self.target_position.map(|t| {
            let dx = t[0] - self.position[0];
            let dy = t[1] - self.position[1];
            let dz = t[2] - self.position[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
    }

    /// Update the AI state machine.
    pub fn update(&mut self, delta: f32, current: [f32; 3]) {
        // State transitions
        self.state = self.next_state();

        // Execute current state
        match self.state {
            SwimAIState::Patrol => self.update_patrol(delta, current),
            SwimAIState::Chase => self.update_chase(delta),
            SwimAIState::Flee => self.update_flee(delta),
            SwimAIState::Returning => self.update_returning(delta),
        }

        // Apply current drift
        self.position[0] += current[0] * delta * 0.3;
        self.position[1] += current[1] * delta * 0.1;
        self.position[2] += current[2] * delta * 0.3;
    }

    /// Determine next state based on current conditions.
    fn next_state(&self) -> SwimAIState {
        let depth = -self.position[1];

        // Check depth range first
        if !self.config.is_in_depth_range(depth) {
            return SwimAIState::Returning;
        }

        // Flee if low health
        if self.health_fraction < self.config.flee_health_threshold
            && self.target_position.is_some()
        {
            return SwimAIState::Flee;
        }

        // Chase if target in range
        if let Some(dist) = self.distance_to_target() {
            if dist < self.config.detection_range && self.health_fraction >= self.config.flee_health_threshold {
                return SwimAIState::Chase;
            }
        }

        SwimAIState::Patrol
    }

    fn update_patrol(&mut self, delta: f32, _current: [f32; 3]) {
        self.patrol_angle += delta * 0.5;
        let speed = self.config.swim_speed;
        self.velocity = [
            speed * self.patrol_angle.cos(),
            speed * 0.1 * (self.patrol_angle * 0.7).sin(),
            speed * self.patrol_angle.sin(),
        ];
        self.apply_velocity(delta);

        // Stay near home
        let dx = self.position[0] - self.config.home_position[0];
        let dz = self.position[2] - self.config.home_position[2];
        let dist_sq = dx * dx + dz * dz;
        if dist_sq > self.config.patrol_radius * self.config.patrol_radius {
            // Steer back toward home
            let dist = dist_sq.sqrt();
            self.velocity[0] -= dx / dist * speed * 0.5;
            self.velocity[2] -= dz / dist * speed * 0.5;
        }
    }

    fn update_chase(&mut self, delta: f32) {
        if let Some(target) = self.target_position {
            let dir = [
                target[0] - self.position[0],
                target[1] - self.position[1],
                target[2] - self.position[2],
            ];
            let dist = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
            if dist > 0.01 {
                let speed = self.config.chase_speed;
                self.velocity = [
                    dir[0] / dist * speed,
                    dir[1] / dist * speed,
                    dir[2] / dist * speed,
                ];
            }
            self.apply_velocity(delta);
        }
    }

    fn update_flee(&mut self, delta: f32) {
        if let Some(target) = self.target_position {
            // Move away from target
            let dir = [
                self.position[0] - target[0],
                self.position[1] - target[1],
                self.position[2] - target[2],
            ];
            let dist = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
            if dist > 0.01 {
                let speed = self.config.flee_speed;
                self.velocity = [
                    dir[0] / dist * speed,
                    dir[1] / dist * speed,
                    dir[2] / dist * speed,
                ];
            }
            self.apply_velocity(delta);
        }
    }

    fn update_returning(&mut self, delta: f32) {
        // Head toward home position at depth range center
        let target_depth = -(self.config.min_depth + self.config.max_depth) / 2.0;
        let target = [
            self.config.home_position[0],
            target_depth,
            self.config.home_position[2],
        ];
        let dir = [
            target[0] - self.position[0],
            target[1] - self.position[1],
            target[2] - self.position[2],
        ];
        let dist = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        if dist > 0.01 {
            let speed = self.config.swim_speed * 1.5;
            self.velocity = [
                dir[0] / dist * speed,
                dir[1] / dist * speed,
                dir[2] / dist * speed,
            ];
        }
        self.apply_velocity(delta);
    }

    fn apply_velocity(&mut self, delta: f32) {
        self.position[0] += self.velocity[0] * delta;
        self.position[1] += self.velocity[1] * delta;
        self.position[2] += self.velocity[2] * delta;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shallow_config() {
        let config = SwimAIConfig::shallow([0.0, -50.0, 0.0]);
        assert!(config.is_in_depth_range(50.0));
        assert!(!config.is_in_depth_range(300.0));
    }

    #[test]
    fn test_mid_config() {
        let config = SwimAIConfig::mid_depth([0.0, -300.0, 0.0]);
        assert!(config.is_in_depth_range(250.0));
        assert!(!config.is_in_depth_range(50.0));
    }

    #[test]
    fn test_deep_config() {
        let config = SwimAIConfig::deep([0.0, -800.0, 0.0]);
        assert!(config.is_in_depth_range(800.0));
        assert!(!config.is_in_depth_range(100.0));
    }

    #[test]
    fn test_starts_patrolling() {
        let config = SwimAIConfig::shallow([0.0, -50.0, 0.0]);
        let ai = SwimAI::new(config, [0.0, -50.0, 0.0]);
        assert_eq!(ai.state(), SwimAIState::Patrol);
    }

    #[test]
    fn test_chase_when_target_nearby() {
        let config = SwimAIConfig::shallow([0.0, -50.0, 0.0]);
        let mut ai = SwimAI::new(config, [0.0, -50.0, 0.0]);
        ai.set_target(Some([5.0, -50.0, 5.0]));
        ai.update(0.1, [0.0, 0.0, 0.0]);
        assert_eq!(ai.state(), SwimAIState::Chase);
    }

    #[test]
    fn test_flee_when_low_health() {
        let config = SwimAIConfig::shallow([0.0, -50.0, 0.0]);
        let mut ai = SwimAI::new(config, [0.0, -50.0, 0.0]);
        ai.set_health(0.1);
        ai.set_target(Some([5.0, -50.0, 5.0]));
        ai.update(0.1, [0.0, 0.0, 0.0]);
        assert_eq!(ai.state(), SwimAIState::Flee);
    }

    #[test]
    fn test_return_when_out_of_depth() {
        let config = SwimAIConfig::shallow([0.0, -50.0, 0.0]);
        let mut ai = SwimAI::new(config, [0.0, 10.0, 0.0]); // Above surface
        ai.update(0.1, [0.0, 0.0, 0.0]);
        assert_eq!(ai.state(), SwimAIState::Returning);
    }

    #[test]
    fn test_patrol_moves() {
        let config = SwimAIConfig::shallow([0.0, -50.0, 0.0]);
        let mut ai = SwimAI::new(config, [0.0, -50.0, 0.0]);
        let start = ai.position();
        ai.update(1.0, [0.0, 0.0, 0.0]);
        let end = ai.position();
        assert!(start != end);
    }

    #[test]
    fn test_current_affects_position() {
        let config = SwimAIConfig::shallow([0.0, -50.0, 0.0]);
        let mut ai = SwimAI::new(config, [0.0, -50.0, 0.0]);
        let pos_before = ai.position();
        // Patrol velocity + current drift
        ai.update(1.0, [10.0, 0.0, 10.0]);
        let pos_after = ai.position();
        // Position should have changed (both from patrol and current)
        assert!(pos_before != pos_after);
    }

    use approx::assert_relative_eq;
}
