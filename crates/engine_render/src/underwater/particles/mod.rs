//! Underwater particle effects.
//!
//! Bubbles, sediment, and plankton particles for underwater environments.
//! Spec 2.4 and 6.1: particle budget max 200 active.

mod bubbles;
mod plankton;
mod sediment;

pub use bubbles::{BubbleEmitter, BubbleParticle};
pub use plankton::{BioluminescenceLevel, PlanktonEmitter, PlanktonParticle};
pub use sediment::{SedimentEmitter, SedimentParticle};

/// Maximum active particles across all systems.
pub const MAX_PARTICLES: usize = 200;

/// General particle trait for underwater effects.
#[derive(Debug, Clone)]
pub struct ParticleState {
    /// World position (x, y, z).
    pub position: [f32; 3],
    /// Velocity (x, y, z) in blocks/sec.
    pub velocity: [f32; 3],
    /// Remaining lifetime in seconds.
    pub lifetime: f32,
    /// Current depth in meters.
    pub depth: f32,
}

impl ParticleState {
    /// Create a new particle state.
    #[must_use]
    pub fn new(position: [f32; 3], velocity: [f32; 3], lifetime: f32, depth: f32) -> Self {
        Self {
            position,
            velocity,
            lifetime,
            depth,
        }
    }

    /// Update position by velocity and decrement lifetime.
    pub fn update(&mut self, delta: f32) {
        self.position[0] += self.velocity[0] * delta;
        self.position[1] += self.velocity[1] * delta;
        self.position[2] += self.velocity[2] * delta;
        self.depth = -self.position[1]; // Y-down = depth
        self.lifetime -= delta;
    }

    /// Check if the particle is still alive.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.lifetime > 0.0
    }
}
