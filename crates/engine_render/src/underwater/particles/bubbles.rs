//! Bubble particle system.
//!
//! Bubbles rise from player, thermal vents, and creature actions.
//! They accelerate slightly as they rise and pop at the surface.

use super::ParticleState;

/// Default rise speed for bubbles (blocks/sec).
pub const BUBBLE_RISE_SPEED: f32 = 1.5;

/// Acceleration factor as bubbles rise.
pub const BUBBLE_ACCELERATION: f32 = 0.3;

/// Default bubble lifetime (seconds).
pub const BUBBLE_LIFETIME: f32 = 8.0;

/// Maximum horizontal drift speed.
pub const BUBBLE_DRIFT: f32 = 0.2;

/// A single bubble particle.
#[derive(Debug, Clone)]
pub struct BubbleParticle {
    pub state: ParticleState,
    /// Visual size (0.05 to 0.2 blocks).
    pub size: f32,
    /// Wobble phase for horizontal drift.
    pub wobble_phase: f32,
}

impl BubbleParticle {
    /// Create a new bubble at a position.
    #[must_use]
    pub fn new(position: [f32; 3], depth: f32) -> Self {
        let size = 0.05 + (0.15 * ((position[0] * 7.3 + position[2] * 3.1).rem_euclid(1.0)));
        Self {
            state: ParticleState::new(
                position,
                [0.0, BUBBLE_RISE_SPEED, 0.0],
                BUBBLE_LIFETIME,
                depth,
            ),
            size,
            wobble_phase: (position[0] * 2.17 + position[2] * 5.31).rem_euclid(6.28),
        }
    }

    /// Create a bubble at a vent position.
    #[must_use]
    pub fn from_vent(vent_pos: [f32; 3], depth: f32) -> Self {
        let offset_x = ((vent_pos[0] * 1.7).rem_euclid(1.0) - 0.5) * 0.5;
        let offset_z = ((vent_pos[2] * 2.3).rem_euclid(1.0) - 0.5) * 0.5;
        let pos = [vent_pos[0] + offset_x, vent_pos[1], vent_pos[2] + offset_z];
        Self::new(pos, depth)
    }

    /// Update bubble physics.
    pub fn update(&mut self, delta: f32) {
        // Rise with slight acceleration
        let rise_speed = BUBBLE_RISE_SPEED + BUBBLE_ACCELERATION * (1.0 - (self.state.depth / 200.0).min(1.0));
        self.state.velocity[1] = rise_speed;

        // Horizontal wobble
        self.wobble_phase += delta * 2.0;
        self.state.velocity[0] = BUBBLE_DRIFT * (self.wobble_phase).sin();
        self.state.velocity[2] = BUBBLE_DRIFT * (self.wobble_phase * 0.7).cos();

        self.state.update(delta);

        // Pop at surface
        if self.state.depth <= 0.0 {
            self.state.lifetime = 0.0;
        }
    }

    /// Check if bubble is still alive.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.state.is_alive() && self.state.depth > 0.0
    }
}

/// Bubble particle emitter.
#[derive(Debug, Clone)]
pub struct BubbleEmitter {
    /// Active bubbles.
    particles: Vec<BubbleParticle>,
    /// Emission rate (bubbles per second).
    emit_rate: f32,
    /// Time since last emission.
    emit_timer: f32,
    /// Whether the emitter is active.
    active: bool,
    /// Fixed emission position (for vents).
    source_position: Option<[f32; 3]>,
}

impl BubbleEmitter {
    /// Create a new bubble emitter.
    #[must_use]
    pub fn new(emit_rate: f32) -> Self {
        Self {
            particles: Vec::new(),
            emit_rate,
            emit_timer: 0.0,
            active: true,
            source_position: None,
        }
    }

    /// Create a vent-fixed emitter.
    #[must_use]
    pub fn new_vent(vent_pos: [f32; 3], emit_rate: f32) -> Self {
        Self {
            particles: Vec::new(),
            emit_rate,
            emit_timer: 0.0,
            active: true,
            source_position: Some(vent_pos),
        }
    }

    /// Set emission rate.
    pub fn set_emit_rate(&mut self, rate: f32) {
        self.emit_rate = rate;
    }

    /// Enable or disable the emitter.
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Check if emitter is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get number of active bubbles.
    #[must_use]
    pub fn count(&self) -> usize {
        self.particles.iter().filter(|b| b.is_alive()).count()
    }

    /// Emit bubbles at a position (for player breathing, etc).
    pub fn emit_at(&mut self, position: [f32; 3], depth: f32) {
        if self.particles.len() < super::MAX_PARTICLES {
            self.particles.push(BubbleParticle::new(position, depth));
        }
    }

    /// Update all bubbles and emit new ones from source position.
    pub fn update(&mut self, delta: f32, player_pos: Option<[f32; 3]>, player_depth: f32) {
        // Update existing
        for bubble in &mut self.particles {
            bubble.update(delta);
        }

        // Remove dead
        self.particles.retain(|b| b.is_alive());

        // Emit from source
        if self.active {
            self.emit_timer += delta;
            let interval = 1.0 / self.emit_rate.max(0.01);
            while self.emit_timer >= interval && self.particles.len() < super::MAX_PARTICLES {
                self.emit_timer -= interval;
                if let Some(pos) = self.source_position {
                    let depth = -pos[1];
                    self.particles.push(BubbleParticle::from_vent(pos, depth));
                } else if let Some(pos) = player_pos {
                    self.particles.push(BubbleParticle::new(pos, player_depth));
                }
            }
        }
    }

    /// Get reference to active particles.
    #[must_use]
    pub fn particles(&self) -> &[BubbleParticle] {
        &self.particles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bubble_creation() {
        let bubble = BubbleParticle::new([10.0, -50.0, 10.0], 50.0);
        assert_relative_eq!(bubble.state.depth, 50.0);
        assert!(bubble.size > 0.0 && bubble.size <= 0.2);
        assert!(bubble.is_alive());
    }

    #[test]
    fn test_bubble_rises() {
        let mut bubble = BubbleParticle::new([10.0, -50.0, 10.0], 50.0);
        let initial_y = bubble.state.position[1];
        bubble.update(1.0);
        assert!(bubble.state.position[1] > initial_y);
    }

    #[test]
    fn test_bubble_pops_at_surface() {
        let mut bubble = BubbleParticle::new([10.0, -0.5, 10.0], 0.5);
        bubble.update(1.0);
        assert!(!bubble.is_alive());
    }

    #[test]
    fn test_bubble_from_vent() {
        let bubble = BubbleParticle::from_vent([50.0, -200.0, 50.0], 200.0);
        // Should be near vent position
        assert!((bubble.state.position[0] - 50.0).abs() < 1.0);
        assert!((bubble.state.position[2] - 50.0).abs() < 1.0);
        assert_relative_eq!(bubble.state.depth, 200.0, max_relative = 1.0);
    }

    #[test]
    fn test_emitter_emits() {
        let mut emitter = BubbleEmitter::new_vent([10.0, -50.0, 10.0], 5.0);
        emitter.update(1.0, None, 0.0);
        assert!(emitter.count() > 0);
    }

    #[test]
    fn test_emitter_respects_max() {
        let mut emitter = BubbleEmitter::new_vent([10.0, -50.0, 10.0], 1000.0);
        emitter.update(1.0, None, 0.0);
        assert!(emitter.count() <= super::super::MAX_PARTICLES);
    }

    use approx::assert_relative_eq;
}
