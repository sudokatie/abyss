//! Underwater audio effects.
//!
//! Low-pass filter with depth-dependent cutoff.
//! Reverb per environment, ambient per zone.

/// Surface audio cutoff frequency (Hz).
pub const SURFACE_CUTOFF: f32 = 20000.0;

/// Deep underwater cutoff frequency (Hz).
pub const DEEP_CUTOFF: f32 = 500.0;

/// Depth where maximum muffling occurs.
pub const MAX_MUFFLE_DEPTH: f32 = 500.0;

/// Reverb environment type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnderwaterEnvironment {
    /// Open water.
    OpenWater,
    /// Inside a cave.
    Cave,
    /// Inside a sealed base.
    BaseInterior,
}

impl UnderwaterEnvironment {
    /// Reverb decay time (seconds).
    #[must_use]
    pub fn reverb_decay(&self) -> f32 {
        match self {
            UnderwaterEnvironment::OpenWater => 1.0,
            UnderwaterEnvironment::Cave => 3.0,
            UnderwaterEnvironment::BaseInterior => 0.5,
        }
    }

    /// Reverb wet/dry mix (0.0 to 1.0).
    #[must_use]
    pub fn reverb_mix(&self) -> f32 {
        match self {
            UnderwaterEnvironment::OpenWater => 0.3,
            UnderwaterEnvironment::Cave => 0.6,
            UnderwaterEnvironment::BaseInterior => 0.2,
        }
    }
}

/// Calculate low-pass filter cutoff at a given depth.
#[must_use]
pub fn cutoff_at_depth(depth: f32) -> f32 {
    if depth <= 0.0 {
        return SURFACE_CUTOFF;
    }
    let depth_frac = (depth / MAX_MUFFLE_DEPTH).min(1.0);
    // Exponential decay of cutoff with depth
    SURFACE_CUTOFF * (1.0 - depth_frac).powi(2) + DEEP_CUTOFF * depth_frac
}

/// Calculate muffling factor (0.0 = no muffling, 1.0 = full muffling).
#[must_use]
pub fn muffle_factor(depth: f32) -> f32 {
    if depth <= 0.0 {
        return 0.0;
    }
    (depth / MAX_MUFFLE_DEPTH).min(1.0)
}

/// Ambient sound type for underwater zones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnderwaterAmbient {
    /// Bubbles rising.
    Bubbles,
    /// Water current.
    Current,
    /// Whale song (rare).
    WhaleSong,
    /// Deep rumble.
    DeepRumble,
    /// Bioluminescent crackle.
    BioCrackle,
    /// Cave drip.
    CaveDrip,
}

impl UnderwaterAmbient {
    /// Depth range where this ambient is heard.
    #[must_use]
    pub fn depth_range(&self) -> (f32, f32) {
        match self {
            UnderwaterAmbient::Bubbles => (0.0, 200.0),
            UnderwaterAmbient::Current => (0.0, 500.0),
            UnderwaterAmbient::WhaleSong => (0.0, 300.0),
            UnderwaterAmbient::DeepRumble => (300.0, 2000.0),
            UnderwaterAmbient::BioCrackle => (200.0, 800.0),
            UnderwaterAmbient::CaveDrip => (50.0, 500.0),
        }
    }

    /// Base volume (0.0 to 1.0).
    #[must_use]
    pub fn base_volume(&self) -> f32 {
        match self {
            UnderwaterAmbient::Bubbles => 0.3,
            UnderwaterAmbient::Current => 0.4,
            UnderwaterAmbient::WhaleSong => 0.2,
            UnderwaterAmbient::DeepRumble => 0.5,
            UnderwaterAmbient::BioCrackle => 0.15,
            UnderwaterAmbient::CaveDrip => 0.25,
        }
    }

    /// Check if this ambient plays at a given depth.
    #[must_use]
    pub fn plays_at_depth(&self, depth: f32) -> bool {
        let (min, max) = self.depth_range();
        depth >= min && depth <= max
    }
}

/// Underwater audio controller.
#[derive(Debug, Clone)]
pub struct UnderwaterAudio {
    /// Current depth.
    pub depth: f32,
    /// Whether player is underwater.
    pub is_underwater: bool,
    /// Current environment.
    pub environment: UnderwaterEnvironment,
    /// Current cutoff frequency.
    pub cutoff: f32,
    /// Current muffle factor.
    pub muffle: f32,
}

impl UnderwaterAudio {
    /// Create new underwater audio controller.
    #[must_use]
    pub fn new() -> Self {
        Self {
            depth: 0.0,
            is_underwater: false,
            environment: UnderwaterEnvironment::OpenWater,
            cutoff: SURFACE_CUTOFF,
            muffle: 0.0,
        }
    }

    /// Update audio based on player state.
    pub fn update(&mut self, depth: f32, is_underwater: bool, environment: UnderwaterEnvironment) {
        self.depth = depth;
        self.is_underwater = is_underwater;
        self.environment = environment;

        if is_underwater {
            self.cutoff = cutoff_at_depth(depth);
            self.muffle = muffle_factor(depth);
        } else {
            self.cutoff = SURFACE_CUTOFF;
            self.muffle = 0.0;
        }
    }

    /// Get active ambient sounds for current depth.
    #[must_use]
    pub fn active_ambients(&self) -> Vec<UnderwaterAmbient> {
        if !self.is_underwater {
            return vec![];
        }
        let all = [
            UnderwaterAmbient::Bubbles,
            UnderwaterAmbient::Current,
            UnderwaterAmbient::WhaleSong,
            UnderwaterAmbient::DeepRumble,
            UnderwaterAmbient::BioCrackle,
            UnderwaterAmbient::CaveDrip,
        ];
        all.iter()
            .filter(|a| a.plays_at_depth(self.depth))
            .copied()
            .collect()
    }
}

impl Default for UnderwaterAudio {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_no_muffle() {
        assert_relative_eq!(muffle_factor(0.0), 0.0);
    }

    #[test]
    fn test_deep_muffle() {
        assert!(muffle_factor(500.0) > 0.9);
    }

    #[test]
    fn test_cutoff_decreases_with_depth() {
        let surface = cutoff_at_depth(0.0);
        let mid = cutoff_at_depth(100.0);
        let deep = cutoff_at_depth(400.0);
        assert!(surface > mid);
        assert!(mid > deep);
    }

    #[test]
    fn test_cave_reverb_longer() {
        assert!(UnderwaterEnvironment::Cave.reverb_decay() > UnderwaterEnvironment::OpenWater.reverb_decay());
    }

    #[test]
    fn test_bubbles_shallow() {
        assert!(UnderwaterAmbient::Bubbles.plays_at_depth(50.0));
        assert!(!UnderwaterAmbient::Bubbles.plays_at_depth(300.0));
    }

    #[test]
    fn test_deep_rumble_deep() {
        assert!(!UnderwaterAmbient::DeepRumble.plays_at_depth(100.0));
        assert!(UnderwaterAmbient::DeepRumble.plays_at_depth(500.0));
    }

    #[test]
    fn test_audio_controller_surface() {
        let mut audio = UnderwaterAudio::new();
        audio.update(0.0, false, UnderwaterEnvironment::OpenWater);
        assert_relative_eq!(audio.muffle, 0.0);
        assert!(audio.active_ambients().is_empty());
    }

    #[test]
    fn test_audio_controller_underwater() {
        let mut audio = UnderwaterAudio::new();
        audio.update(100.0, true, UnderwaterEnvironment::OpenWater);
        assert!(audio.muffle > 0.0);
        assert!(!audio.active_ambients().is_empty());
    }

    use approx::assert_relative_eq;
}
