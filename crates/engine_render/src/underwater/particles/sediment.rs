//! Sediment particle system.
//!
//! Kicked up by player movement, settles over time. Creates dust
//! clouds near the ocean floor.

use super::ParticleState;

/// Default sediment lifetime (seconds).
pub const SEDIMENT_LIFETIME: f32 = 4.0;

/// Settling speed (blocks/sec downward).
pub const SEDIMENT_SETTLE_SPEED: f32 = 0.3;

/// Horizontal spread speed.
pub const SEDIMENT_SPREAD: f32 = 0.5;

/// Default emission per movement event.
pub const SEDIMENT_PER_KICK: usize = 5;

/// A single sediment particle.
#[derive(Debug, Clone)]
pub struct SedimentParticle {
    pub state: ParticleState,
    /// Opacity (fades as it settles).
    pub opacity: f32,
}

impl SedimentParticle {
    /// Create sediment from a kick at a position.
    #[must_use]
    pub fn from_kick(position: [f32; 3], depth: f32) -> Self {
        // Random-ish spread based on position hash
        let hash = ((position[0] * 17.3 + position[2] * 31.7).rem_euclid(1.0));
        let vx = (hash - 0.5) * SEDIMENT_SPREAD * 2.0;
        let vz = ((hash * 7.13).rem_euclid(1.0) - 0.5) * SEDIMENT_SPREAD * 2.0;
        let vy = 0.2 + (hash * 0.3); // slight upward kick

        Self {
            state: ParticleState::new(position, [vx, vy, vz], SEDIMENT_LIFETIME, depth),
            opacity: 0.8,
        }
    }

    /// Create sediment near ocean floor.
    #[must_use]
    pub fn from_floor(position: [f32; 3], depth: f32) -> Self {
        let hash = (position[0] * 11.7 + position[2] * 23.1).rem_euclid(1.0);
        let vx = (hash - 0.5) * SEDIMENT_SPREAD;
        let vz = ((hash * 3.7).rem_euclid(1.0) - 0.5) * SEDIMENT_SPREAD;

        Self {
            state: ParticleState::new(
                position,
                [vx, SEDIMENT_SETTLE_SPEED, vz],
                SEDIMENT_LIFETIME,
                depth,
            ),
            opacity: 0.5,
        }
    }

    /// Update sediment physics.
    pub fn update(&mut self, delta: f32) {
        // Settle downward over time
        let settle_progress = 1.0 - (self.state.lifetime / SEDIMENT_LIFETIME).min(1.0);
        self.state.velocity[1] = -SEDIMENT_SETTLE_SPEED * settle_progress;

        // Reduce horizontal speed as it settles
        let dampening = 1.0 - settle_progress * 0.5;
        self.state.velocity[0] *= dampening;
        self.state.velocity[2] *= dampening;

        self.state.update(delta);

        // Fade opacity
        self.opacity = 0.8 * (self.state.lifetime / SEDIMENT_LIFETIME).min(1.0);
    }

    /// Check if sediment is still alive.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.state.is_alive()
    }
}

/// Sediment particle emitter.
#[derive(Debug, Clone)]
pub struct SedimentEmitter {
    /// Active sediment particles.
    particles: Vec<SedimentParticle>,
    /// Whether the emitter is enabled.
    active: bool,
}

impl SedimentEmitter {
    /// Create a new sediment emitter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            active: true,
        }
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

    /// Get number of active particles.
    #[must_use]
    pub fn count(&self) -> usize {
        self.particles.iter().filter(|p| p.is_alive()).count()
    }

    /// Kick up sediment at a position (from player movement).
    pub fn kick(&mut self, position: [f32; 3], depth: f32) {
        if !self.active {
            return;
        }
        let budget = super::MAX_PARTICLES.saturating_sub(self.particles.len());
        let count = SEDIMENT_PER_KICK.min(budget);
        for i in 0..count {
            let offset = (i as f32 * 0.1).rem_euclid(1.0);
            let pos = [position[0] + offset, position[1], position[2] + offset * 0.7];
            self.particles.push(SedimentParticle::from_kick(pos, depth));
        }
    }

    /// Emit floor sediment (ambient dust near ocean floor).
    pub fn emit_floor(&mut self, position: [f32; 3], depth: f32) {
        if !self.active || self.particles.len() >= super::MAX_PARTICLES {
            return;
        }
        self.particles.push(SedimentParticle::from_floor(position, depth));
    }

    /// Update all sediment particles.
    pub fn update(&mut self, delta: f32) {
        for particle in &mut self.particles {
            particle.update(delta);
        }
        self.particles.retain(|p| p.is_alive());
    }

    /// Get reference to active particles.
    #[must_use]
    pub fn particles(&self) -> &[SedimentParticle] {
        &self.particles
    }
}

impl Default for SedimentEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sediment_from_kick() {
        let sediment = SedimentParticle::from_kick([10.0, -50.0, 10.0], 50.0);
        assert!(sediment.is_alive());
        assert_relative_eq!(sediment.opacity, 0.8);
    }

    #[test]
    fn test_sediment_settles() {
        let mut sediment = SedimentParticle::from_kick([10.0, -50.0, 10.0], 50.0);
        let initial_y = sediment.state.position[1];
        sediment.update(2.0);
        // After settling, should be moving downward
        assert!(sediment.state.velocity[1] < 0.0 || sediment.state.position[1] <= initial_y + 0.5);
    }

    #[test]
    fn test_sediment_fades() {
        let mut sediment = SedimentParticle::from_kick([10.0, -50.0, 10.0], 50.0);
        sediment.update(3.0);
        assert!(sediment.opacity < 0.8);
    }

    #[test]
    fn test_emitter_kick() {
        let mut emitter = SedimentEmitter::new();
        emitter.kick([10.0, -50.0, 10.0], 50.0);
        assert_eq!(emitter.count(), SEDIMENT_PER_KICK);
    }

    #[test]
    fn test_emitter_respects_max() {
        let mut emitter = SedimentEmitter::new();
        for _ in 0..50 {
            emitter.kick([10.0, -50.0, 10.0], 50.0);
        }
        assert!(emitter.count() <= super::super::MAX_PARTICLES);
    }

    use approx::assert_relative_eq;
}
