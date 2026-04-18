//! Performance profiling module for the Abyss game.
//!
//! Provides tools for tracking system update times, frame rates,
//! and checking against performance budgets.

use std::collections::HashMap;
use std::time::Instant;

/// Names of systems that can be profiled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemName {
    Oxygen,
    Pressure,
    Creatures,
    Particles,
    AI,
    WorldGeneration,
    Network,
    Rendering,
    Physics,
    Audio,
}

impl SystemName {
    /// Returns all system names.
    pub fn all() -> &'static [SystemName] {
        &[
            SystemName::Oxygen,
            SystemName::Pressure,
            SystemName::Creatures,
            SystemName::Particles,
            SystemName::AI,
            SystemName::WorldGeneration,
            SystemName::Network,
            SystemName::Rendering,
            SystemName::Physics,
            SystemName::Audio,
        ]
    }
}

/// Types of performance budget violations.
#[derive(Debug, Clone, PartialEq)]
pub enum BudgetViolation {
    /// Frame time exceeded the target (actual_ms, target_ms).
    FrameTimeExceeded { actual_ms: f64, target_ms: f64 },
    /// Particle count exceeded the limit (actual, max).
    ParticleLimitExceeded { actual: u32, max: u32 },
    /// Creature count exceeded the limit (actual, max).
    CreatureLimitExceeded { actual: u32, max: u32 },
    /// Chunk generation took too long (actual_ms, budget_ms).
    ChunkGenSlow { actual_ms: f64, budget_ms: f64 },
}

/// Tracks individual system update times.
pub struct SystemProfiler {
    /// Start times for currently running systems.
    start_times: HashMap<SystemName, Instant>,
    /// Accumulated times for each system in the current frame (in seconds).
    system_times: HashMap<SystemName, f64>,
    /// Frame start time.
    frame_start: Option<Instant>,
    /// Total frame time in seconds.
    frame_time: f64,
}

impl Default for SystemProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemProfiler {
    /// Creates a new system profiler.
    pub fn new() -> Self {
        Self {
            start_times: HashMap::new(),
            system_times: HashMap::new(),
            frame_start: None,
            frame_time: 0.0,
        }
    }

    /// Starts the frame timer.
    pub fn begin_frame(&mut self) {
        self.frame_start = Some(Instant::now());
        self.system_times.clear();
        self.start_times.clear();
    }

    /// Ends the frame timer and records total frame time.
    pub fn end_frame(&mut self) {
        if let Some(start) = self.frame_start.take() {
            self.frame_time = start.elapsed().as_secs_f64();
        }
    }

    /// Starts timing a system.
    pub fn start(&mut self, system: SystemName) {
        self.start_times.insert(system, Instant::now());
    }

    /// Ends timing a system and records the elapsed time.
    pub fn end(&mut self, system: SystemName) {
        if let Some(start) = self.start_times.remove(&system) {
            let elapsed = start.elapsed().as_secs_f64();
            *self.system_times.entry(system).or_insert(0.0) += elapsed;
        }
    }

    /// Gets the recorded time for a system in milliseconds.
    pub fn get_time_ms(&self, system: SystemName) -> f64 {
        self.system_times.get(&system).copied().unwrap_or(0.0) * 1000.0
    }

    /// Gets the total frame time in milliseconds.
    pub fn frame_time_ms(&self) -> f64 {
        self.frame_time * 1000.0
    }

    /// Generates a performance report.
    pub fn report(&self) -> PerformanceReport {
        let per_system: HashMap<SystemName, f64> = self
            .system_times
            .iter()
            .map(|(&k, &v)| (k, v * 1000.0))
            .collect();

        let total_frame_ms = self.frame_time * 1000.0;
        let fps = if self.frame_time > 0.0 {
            1.0 / self.frame_time
        } else {
            0.0
        };

        PerformanceReport {
            per_system,
            total_frame_ms,
            fps,
        }
    }
}

/// Tracks frame times and calculates FPS.
pub struct FrameTimer {
    /// Rolling buffer of frame times.
    frame_times: Vec<f64>,
    /// Maximum number of frames to track.
    max_frames: usize,
    /// Current index in the circular buffer.
    index: usize,
    /// Number of frames recorded.
    count: usize,
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::new(60)
    }
}

impl FrameTimer {
    /// Creates a new frame timer with the specified window size.
    pub fn new(window_size: usize) -> Self {
        Self {
            frame_times: vec![0.0; window_size],
            max_frames: window_size,
            index: 0,
            count: 0,
        }
    }

    /// Records a frame with the given delta time in seconds.
    pub fn frame(&mut self, delta: f64) {
        self.frame_times[self.index] = delta;
        self.index = (self.index + 1) % self.max_frames;
        if self.count < self.max_frames {
            self.count += 1;
        }
    }

    /// Returns the rolling average FPS.
    pub fn fps(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }

        let sum: f64 = self.frame_times.iter().take(self.count).sum();
        if sum > 0.0 {
            self.count as f64 / sum
        } else {
            0.0
        }
    }

    /// Returns the average frame time in milliseconds.
    pub fn average_frame_time_ms(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }

        let sum: f64 = self.frame_times.iter().take(self.count).sum();
        (sum / self.count as f64) * 1000.0
    }

    /// Returns the number of frames recorded.
    pub fn frame_count(&self) -> usize {
        self.count
    }
}

/// Performance budget constraints.
#[derive(Debug, Clone)]
pub struct PerformanceBudget {
    /// Target frames per second.
    pub target_fps: f64,
    /// Maximum particle count.
    pub max_particle_count: u32,
    /// Maximum creature count.
    pub max_creatures: u32,
    /// Maximum player count.
    pub max_players: u32,
    /// Maximum time budget for chunk generation in milliseconds.
    pub chunk_gen_budget_ms: f64,
}

impl Default for PerformanceBudget {
    fn default() -> Self {
        Self {
            target_fps: 60.0,
            max_particle_count: 200,
            max_creatures: 50,
            max_players: 10,
            chunk_gen_budget_ms: 50.0,
        }
    }
}

impl PerformanceBudget {
    /// Creates a new performance budget with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Target frame time in milliseconds based on target FPS.
    pub fn target_frame_time_ms(&self) -> f64 {
        if self.target_fps > 0.0 {
            1000.0 / self.target_fps
        } else {
            0.0
        }
    }

    /// Checks a performance report against the budget.
    pub fn check(&self, report: &PerformanceReport) -> Vec<BudgetViolation> {
        let mut violations = Vec::new();

        let target_ms = self.target_frame_time_ms();
        if report.total_frame_ms > target_ms {
            violations.push(BudgetViolation::FrameTimeExceeded {
                actual_ms: report.total_frame_ms,
                target_ms,
            });
        }

        violations
    }

    /// Checks a performance report with additional runtime metrics.
    pub fn check_with_metrics(
        &self,
        report: &PerformanceReport,
        particle_count: u32,
        creature_count: u32,
    ) -> Vec<BudgetViolation> {
        let mut violations = self.check(report);

        if particle_count > self.max_particle_count {
            violations.push(BudgetViolation::ParticleLimitExceeded {
                actual: particle_count,
                max: self.max_particle_count,
            });
        }

        if creature_count > self.max_creatures {
            violations.push(BudgetViolation::CreatureLimitExceeded {
                actual: creature_count,
                max: self.max_creatures,
            });
        }

        if let Some(&chunk_gen_ms) = report.per_system.get(&SystemName::WorldGeneration) {
            if chunk_gen_ms > self.chunk_gen_budget_ms {
                violations.push(BudgetViolation::ChunkGenSlow {
                    actual_ms: chunk_gen_ms,
                    budget_ms: self.chunk_gen_budget_ms,
                });
            }
        }

        violations
    }
}

/// A snapshot of performance metrics.
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// Time spent in each system (in milliseconds).
    pub per_system: HashMap<SystemName, f64>,
    /// Total frame time in milliseconds.
    pub total_frame_ms: f64,
    /// Frames per second.
    pub fps: f64,
}

impl PerformanceReport {
    /// Checks if the frame time meets a target FPS budget.
    pub fn meets_fps_target(&self, target_fps: f64) -> bool {
        if target_fps <= 0.0 {
            return true;
        }
        let target_ms = 1000.0 / target_fps;
        self.total_frame_ms <= target_ms
    }

    /// Gets the time for a specific system in milliseconds.
    pub fn system_time_ms(&self, system: SystemName) -> f64 {
        self.per_system.get(&system).copied().unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_system_profiler_basic() {
        let mut profiler = SystemProfiler::new();
        profiler.begin_frame();
        profiler.start(SystemName::Oxygen);
        thread::sleep(Duration::from_millis(5));
        profiler.end(SystemName::Oxygen);
        profiler.end_frame();

        assert!(profiler.get_time_ms(SystemName::Oxygen) >= 4.0);
    }

    #[test]
    fn test_system_profiler_multiple_systems() {
        let mut profiler = SystemProfiler::new();
        profiler.begin_frame();

        profiler.start(SystemName::Oxygen);
        thread::sleep(Duration::from_millis(2));
        profiler.end(SystemName::Oxygen);

        profiler.start(SystemName::Pressure);
        thread::sleep(Duration::from_millis(2));
        profiler.end(SystemName::Pressure);

        profiler.end_frame();

        assert!(profiler.get_time_ms(SystemName::Oxygen) >= 1.0);
        assert!(profiler.get_time_ms(SystemName::Pressure) >= 1.0);
        assert!(profiler.frame_time_ms() >= 3.0);
    }

    #[test]
    fn test_system_profiler_report() {
        let mut profiler = SystemProfiler::new();
        profiler.begin_frame();
        profiler.start(SystemName::Creatures);
        thread::sleep(Duration::from_millis(3));
        profiler.end(SystemName::Creatures);
        profiler.end_frame();

        let report = profiler.report();
        assert!(report.per_system.contains_key(&SystemName::Creatures));
        assert!(report.total_frame_ms >= 2.0);
        assert!(report.fps > 0.0);
    }

    #[test]
    fn test_frame_timer_basic() {
        let mut timer = FrameTimer::new(60);
        timer.frame(1.0 / 60.0);
        timer.frame(1.0 / 60.0);
        timer.frame(1.0 / 60.0);

        assert_eq!(timer.frame_count(), 3);
        let fps = timer.fps();
        assert!((fps - 60.0).abs() < 1.0);
    }

    #[test]
    fn test_frame_timer_rolling_average() {
        let mut timer = FrameTimer::new(4);

        // Fill the buffer
        timer.frame(1.0 / 30.0); // 30 FPS
        timer.frame(1.0 / 30.0);
        timer.frame(1.0 / 30.0);
        timer.frame(1.0 / 30.0);

        assert!((timer.fps() - 30.0).abs() < 1.0);

        // Overwrite with faster frames
        timer.frame(1.0 / 60.0); // 60 FPS
        timer.frame(1.0 / 60.0);
        timer.frame(1.0 / 60.0);
        timer.frame(1.0 / 60.0);

        assert!((timer.fps() - 60.0).abs() < 1.0);
    }

    #[test]
    fn test_frame_timer_average_frame_time() {
        let mut timer = FrameTimer::new(10);
        for _ in 0..10 {
            timer.frame(0.016); // ~60 FPS
        }

        let avg_ms = timer.average_frame_time_ms();
        assert!((avg_ms - 16.0).abs() < 0.1);
    }

    #[test]
    fn test_performance_budget_defaults() {
        let budget = PerformanceBudget::default();
        assert_eq!(budget.target_fps, 60.0);
        assert_eq!(budget.max_particle_count, 200);
        assert_eq!(budget.max_creatures, 50);
        assert_eq!(budget.max_players, 10);
        assert_eq!(budget.chunk_gen_budget_ms, 50.0);
    }

    #[test]
    fn test_budget_frame_time_violation() {
        let budget = PerformanceBudget::default();
        let report = PerformanceReport {
            per_system: HashMap::new(),
            total_frame_ms: 20.0, // 50 FPS, under 60 FPS target
            fps: 50.0,
        };

        let violations = budget.check(&report);
        assert_eq!(violations.len(), 1);
        match &violations[0] {
            BudgetViolation::FrameTimeExceeded { actual_ms, target_ms } => {
                assert_eq!(*actual_ms, 20.0);
                assert!((target_ms - 16.666).abs() < 0.01);
            }
            _ => panic!("Expected FrameTimeExceeded violation"),
        }
    }

    #[test]
    fn test_budget_no_violations() {
        let budget = PerformanceBudget::default();
        let report = PerformanceReport {
            per_system: HashMap::new(),
            total_frame_ms: 10.0, // 100 FPS, above 60 FPS target
            fps: 100.0,
        };

        let violations = budget.check(&report);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_budget_particle_violation() {
        let budget = PerformanceBudget::default();
        let report = PerformanceReport {
            per_system: HashMap::new(),
            total_frame_ms: 10.0,
            fps: 100.0,
        };

        let violations = budget.check_with_metrics(&report, 300, 30);
        assert_eq!(violations.len(), 1);
        match &violations[0] {
            BudgetViolation::ParticleLimitExceeded { actual, max } => {
                assert_eq!(*actual, 300);
                assert_eq!(*max, 200);
            }
            _ => panic!("Expected ParticleLimitExceeded violation"),
        }
    }

    #[test]
    fn test_budget_creature_violation() {
        let budget = PerformanceBudget::default();
        let report = PerformanceReport {
            per_system: HashMap::new(),
            total_frame_ms: 10.0,
            fps: 100.0,
        };

        let violations = budget.check_with_metrics(&report, 100, 75);
        assert_eq!(violations.len(), 1);
        match &violations[0] {
            BudgetViolation::CreatureLimitExceeded { actual, max } => {
                assert_eq!(*actual, 75);
                assert_eq!(*max, 50);
            }
            _ => panic!("Expected CreatureLimitExceeded violation"),
        }
    }

    #[test]
    fn test_budget_chunk_gen_violation() {
        let budget = PerformanceBudget::default();
        let mut per_system = HashMap::new();
        per_system.insert(SystemName::WorldGeneration, 75.0);

        let report = PerformanceReport {
            per_system,
            total_frame_ms: 10.0,
            fps: 100.0,
        };

        let violations = budget.check_with_metrics(&report, 100, 30);
        assert_eq!(violations.len(), 1);
        match &violations[0] {
            BudgetViolation::ChunkGenSlow { actual_ms, budget_ms } => {
                assert_eq!(*actual_ms, 75.0);
                assert_eq!(*budget_ms, 50.0);
            }
            _ => panic!("Expected ChunkGenSlow violation"),
        }
    }

    #[test]
    fn test_report_meets_fps_target() {
        let fast_report = PerformanceReport {
            per_system: HashMap::new(),
            total_frame_ms: 10.0,
            fps: 100.0,
        };
        assert!(fast_report.meets_fps_target(60.0));

        let slow_report = PerformanceReport {
            per_system: HashMap::new(),
            total_frame_ms: 25.0,
            fps: 40.0,
        };
        assert!(!slow_report.meets_fps_target(60.0));
    }

    #[test]
    fn test_system_name_all() {
        let all = SystemName::all();
        assert_eq!(all.len(), 10);
        assert!(all.contains(&SystemName::Oxygen));
        assert!(all.contains(&SystemName::Physics));
        assert!(all.contains(&SystemName::Audio));
    }
}
