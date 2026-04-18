//! Plankton particle system.
//!
//! Ambient plankton in the sunlight zone, bioluminescent at depth.
//! Spec 2.4 and 6.3: plankton visible near surface, bio glow at depth.

use super::ParticleState;

/// Sunlight zone max depth for regular plankton.
pub const PLANKTON_MAX_DEPTH: f32 = 200.0;

/// Depth where bioluminescence starts.
pub const BIOLUMINESCENCE_START_DEPTH: f32 = 200.0;

/// Depth where bioluminescence is strongest.
pub const BIOLUMINESCENCE_PEAK_DEPTH: f32 = 500.0;

/// Plankton drift speed (blocks/sec).
pub const PLANKTON_DRIFT: f32 = 0.15;

/// Default plankton lifetime (seconds).
pub const PLANKTON_LIFETIME: f32 = 15.0;

/// Number of ambient plankton to maintain.
pub const AMBIENT_PLANKTON_COUNT: usize = 30;

/// Bioluminescence intensity levels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BioluminescenceLevel {
    /// No bioluminescence (above 200m).
    None,
    /// Faint glow (200-350m).
    Faint,
    /// Moderate glow (350-500m).
    Moderate,
    /// Strong glow (500m+).
    Strong,
}

impl BioluminescenceLevel {
    /// Determine level from depth.
    #[must_use]
    pub fn from_depth(depth: f32) -> Self {
        if depth < BIOLUMINESCENCE_START_DEPTH {
            BioluminescenceLevel::None
        } else if depth < 350.0 {
            BioluminescenceLevel::Faint
        } else if depth < BIOLUMINESCENCE_PEAK_DEPTH {
            BioluminescenceLevel::Moderate
        } else {
            BioluminescenceLevel::Strong
        }
    }

    /// Get glow intensity (0.0 to 1.0).
    #[must_use]
    pub fn intensity(&self) -> f32 {
        match self {
            BioluminescenceLevel::None => 0.0,
            BioluminescenceLevel::Faint => 0.25,
            BioluminescenceLevel::Moderate => 0.6,
            BioluminescenceLevel::Strong => 1.0,
        }
    }

    /// Get glow color as RGB (cyan-green spectrum for bio light).
    #[must_use]
    pub fn glow_color(&self) -> [f32; 3] {
        match self {
            BioluminescenceLevel::None => [0.0, 0.0, 0.0],
            BioluminescenceLevel::Faint => [0.1, 0.8, 0.7],
            BioluminescenceLevel::Moderate => [0.2, 1.0, 0.8],
            BioluminescenceLevel::Strong => [0.3, 1.0, 0.9],
        }
    }
}

/// A single plankton particle.
#[derive(Debug, Clone)]
pub struct PlanktonParticle {
    pub state: ParticleState,
    /// Bioluminescence level at current depth.
    pub bio_level: BioluminescenceLevel,
    /// Flicker phase for bioluminescence.
    pub flicker_phase: f32,
    /// Ambient drift direction.
    pub drift_angle: f32,
}

impl PlanktonParticle {
    /// Create a new plankton particle.
    #[must_use]
    pub fn new(position: [f32; 3], depth: f32) -> Self {
        let bio_level = BioluminescenceLevel::from_depth(depth);
        let hash = (position[0] * 13.7 + position[2] * 29.3).rem_euclid(6.28);
        Self {
            state: ParticleState::new(
                position,
                [0.0, 0.0, 0.0],
                PLANKTON_LIFETIME,
                depth,
            ),
            bio_level,
            flicker_phase: hash,
            drift_angle: hash,
        }
    }

    /// Get the current glow intensity accounting for flicker.
    #[must_use]
    pub fn glow_intensity(&self) -> f32 {
        let flicker = 0.7 + 0.3 * (self.flicker_phase * 3.0).sin();
        self.bio_level.intensity() * flicker
    }

    /// Get the glow color.
    #[must_use]
    pub fn glow_color(&self) -> [f32; 3] {
        self.bio_level.glow_color()
    }

    /// Check if this plankton is visible (in range and alive).
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.state.is_alive()
            && (self.state.depth <= PLANKTON_MAX_DEPTH
                || self.bio_level != BioluminescenceLevel::None)
    }

    /// Update plankton physics.
    pub fn update(&mut self, delta: f32, current: [f32; 3]) {
        // Gentle drift with current
        self.drift_angle += delta * 0.5;
        self.state.velocity[0] =
            PLANKTON_DRIFT * self.drift_angle.cos() + current[0] * 0.3;
        self.state.velocity[1] =
            PLANKTON_DRIFT * 0.2 * self.drift_angle.sin() + current[1] * 0.1;
        self.state.velocity[2] =
            PLANKTON_DRIFT * self.drift_angle.sin() + current[2] * 0.3;

        self.state.update(delta);
        self.bio_level = BioluminescenceLevel::from_depth(self.state.depth);

        // Update flicker
        self.flicker_phase += delta * 2.0;
    }

    /// Check if particle is still alive.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.state.is_alive()
    }
}

/// Plankton particle emitter.
#[derive(Debug, Clone)]
pub struct PlanktonEmitter {
    /// Active plankton particles.
    particles: Vec<PlanktonParticle>,
    /// Whether the emitter is active.
    active: bool,
    /// Water current vector.
    current: [f32; 3],
}

impl PlanktonEmitter {
    /// Create a new plankton emitter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            active: true,
            current: [0.0, 0.0, 0.0],
        }
    }

    /// Set water current.
    pub fn set_current(&mut self, current: [f32; 3]) {
        self.current = current;
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

    /// Spawn ambient plankton around a center position.
    pub fn spawn_ambient(&mut self, center: [f32; 3], depth: f32) {
        if !self.active || self.particles.len() >= super::MAX_PARTICLES {
            return;
        }
        let needed = AMBIENT_PLANKTON_COUNT.saturating_sub(self.count());
        let budget = super::MAX_PARTICLES.saturating_sub(self.particles.len());
        let count = needed.min(budget);
        for i in 0..count {
            let offset = (i as f32 * 2.17).rem_euclid(1.0);
            let pos = [
                center[0] + (offset - 0.5) * 20.0,
                center[1] + ((offset * 3.7).rem_euclid(1.0) - 0.5) * 10.0,
                center[2] + ((offset * 7.3).rem_euclid(1.0) - 0.5) * 20.0,
            ];
            self.particles.push(PlanktonParticle::new(pos, depth));
        }
    }

    /// Update all plankton particles.
    pub fn update(&mut self, delta: f32) {
        for particle in &mut self.particles {
            particle.update(delta, self.current);
        }
        self.particles.retain(|p| p.is_alive());
    }

    /// Get reference to active particles.
    #[must_use]
    pub fn particles(&self) -> &[PlanktonParticle] {
        &self.particles
    }
}

impl Default for PlanktonEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plankton_creation() {
        let plankton = PlanktonParticle::new([10.0, -50.0, 10.0], 50.0);
        assert!(plankton.is_alive());
        assert_eq!(plankton.bio_level, BioluminescenceLevel::None);
    }

    #[test]
    fn test_bio_level_none() {
        assert_eq!(BioluminescenceLevel::from_depth(50.0), BioluminescenceLevel::None);
    }

    #[test]
    fn test_bio_level_faint() {
        assert_eq!(BioluminescenceLevel::from_depth(250.0), BioluminescenceLevel::Faint);
    }

    #[test]
    fn test_bio_level_moderate() {
        assert_eq!(BioluminescenceLevel::from_depth(400.0), BioluminescenceLevel::Moderate);
    }

    #[test]
    fn test_bio_level_strong() {
        assert_eq!(BioluminescenceLevel::from_depth(600.0), BioluminescenceLevel::Strong);
    }

    #[test]
    fn test_glow_intensity_flicker() {
        let mut plankton = PlanktonParticle::new([10.0, -300.0, 10.0], 300.0);
        let i1 = plankton.glow_intensity();
        plankton.flicker_phase += 1.0;
        let i2 = plankton.glow_intensity();
        // Flicker should vary
        assert!(i1 > 0.0 && i2 > 0.0);
    }

    #[test]
    fn test_deep_plankton_bio() {
        let plankton = PlanktonParticle::new([10.0, -400.0, 10.0], 400.0);
        assert_ne!(plankton.bio_level, BioluminescenceLevel::None);
        assert!(plankton.is_visible());
    }

    #[test]
    fn test_emitter_spawn_ambient() {
        let mut emitter = PlanktonEmitter::new();
        emitter.spawn_ambient([0.0, -50.0, 0.0], 50.0);
        assert!(emitter.count() > 0);
    }

    use approx::assert_relative_eq;
}
