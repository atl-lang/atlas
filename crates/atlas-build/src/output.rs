//! Build output formatting and progress reporting
//!
//! Provides progress tracking, colorized output, and build summaries.

use crate::cache::CacheStats;
use crate::targets::BuildArtifact;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Build progress tracker
pub struct BuildProgress {
    /// Total number of modules to compile
    total_modules: usize,
    /// Number of modules compiled so far
    compiled_modules: usize,
    /// Current module being compiled
    current_module: Option<String>,
    /// Build start time
    start_time: Instant,
    /// Average compile time per module
    avg_compile_time: Option<Duration>,
    /// Output mode
    mode: OutputMode,
}

impl BuildProgress {
    /// Create new progress tracker
    pub fn new(total_modules: usize, mode: OutputMode) -> Self {
        Self {
            total_modules,
            compiled_modules: 0,
            current_module: None,
            start_time: Instant::now(),
            avg_compile_time: None,
            mode,
        }
    }

    /// Update progress with newly compiled module
    pub fn update(&mut self, module_name: String, compile_time: Duration) {
        self.compiled_modules += 1;
        self.current_module = Some(module_name);

        // Update average compile time
        if let Some(avg) = self.avg_compile_time {
            self.avg_compile_time = Some((avg + compile_time) / 2);
        } else {
            self.avg_compile_time = Some(compile_time);
        }
    }

    /// Start compiling a module
    pub fn start_module(&mut self, module_name: String) {
        self.current_module = Some(module_name);
    }

    /// Report current progress
    pub fn report(&self) {
        if !self.should_report() {
            return;
        }

        if self.total_modules == 0 {
            return;
        }

        let percent = (self.compiled_modules as f64 / self.total_modules as f64) * 100.0;
        let elapsed = self.start_time.elapsed();
        let eta = self.estimate_remaining_time();

        if let Some(ref module) = self.current_module {
            print!(
                "\rCompiling {} ({}/{}) [{:.1}%]",
                module, self.compiled_modules, self.total_modules, percent
            );

            if let Some(eta) = eta {
                if eta.as_secs() > 0 {
                    print!(" - ETA: {:.1}s", eta.as_secs_f64());
                }
            }

            // Clear to end of line and flush
            print!("{}   ", " ".repeat(20));
            use std::io::{self, Write};
            let _ = io::stdout().flush();
        } else {
            println!(
                "Compiled {}/{} modules ({:.1}%) in {:.2}s",
                self.compiled_modules,
                self.total_modules,
                percent,
                elapsed.as_secs_f64()
            );
        }
    }

    /// Estimate remaining build time
    fn estimate_remaining_time(&self) -> Option<Duration> {
        if let Some(avg) = self.avg_compile_time {
            if self.compiled_modules > 0 {
                let remaining = self.total_modules.saturating_sub(self.compiled_modules);
                return Some(avg * remaining as u32);
            }
        }
        None
    }

    /// Check if should report progress
    fn should_report(&self) -> bool {
        !matches!(self.mode, OutputMode::Quiet | OutputMode::Json)
    }

    /// Finish progress reporting
    pub fn finish(&self) {
        if self.should_report() {
            println!(); // New line after progress
        }
    }
}

/// Build summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSummary {
    /// Total build time
    pub total_time: Duration,
    /// Time spent compiling
    pub compile_time: Duration,
    /// Time spent linking
    pub link_time: Duration,
    /// Number of modules
    pub module_count: usize,
    /// Cache hit rate (0.0-1.0)
    pub cache_hit_rate: f64,
    /// Build artifacts produced
    pub artifacts: Vec<BuildArtifact>,
}

impl BuildSummary {
    /// Create new build summary
    pub fn new() -> Self {
        Self {
            total_time: Duration::ZERO,
            compile_time: Duration::ZERO,
            link_time: Duration::ZERO,
            module_count: 0,
            cache_hit_rate: 0.0,
            artifacts: Vec::new(),
        }
    }

    /// Create from cache stats
    pub fn from_cache_stats(cache_stats: &CacheStats, total_time: Duration) -> Self {
        Self {
            total_time,
            compile_time: Duration::ZERO,
            link_time: Duration::ZERO,
            module_count: cache_stats.total_modules,
            cache_hit_rate: cache_stats.cache_hit_rate,
            artifacts: Vec::new(),
        }
    }

    /// Display summary in human-readable format
    pub fn display(&self, mode: &OutputMode) {
        match mode {
            OutputMode::Normal | OutputMode::Verbose => {
                println!("\n{}", "=".repeat(60));
                println!("Build succeeded in {:.2}s", self.total_time.as_secs_f64());
                println!("{}", "=".repeat(60));
                println!(
                    "  {} modules compiled ({} from cache)",
                    self.module_count,
                    (self.module_count as f64 * self.cache_hit_rate) as usize
                );
                if self.cache_hit_rate > 0.0 {
                    println!("  Cache hit rate: {:.1}%", self.cache_hit_rate * 100.0);
                }
                println!("  Artifacts: {}", self.artifacts.len());
                for artifact in &self.artifacts {
                    println!(
                        "    - {:?}: {}",
                        artifact.target.kind,
                        artifact.output_path.display()
                    );
                }
                println!("{}", "=".repeat(60));
            }
            OutputMode::Quiet => {
                // Quiet mode - only show success
                println!("Build succeeded");
            }
            OutputMode::Json => {
                // JSON output handled separately
            }
        }
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct JsonSummary {
            success: bool,
            total_time: f64,
            compile_time: f64,
            link_time: f64,
            modules: usize,
            cache_hit_rate: f64,
            artifacts: Vec<JsonArtifact>,
        }

        #[derive(Serialize)]
        struct JsonArtifact {
            target: String,
            path: String,
        }

        let summary = JsonSummary {
            success: true,
            total_time: self.total_time.as_secs_f64(),
            compile_time: self.compile_time.as_secs_f64(),
            link_time: self.link_time.as_secs_f64(),
            modules: self.module_count,
            cache_hit_rate: self.cache_hit_rate,
            artifacts: self
                .artifacts
                .iter()
                .map(|a| JsonArtifact {
                    target: format!("{:?}", a.target.kind),
                    path: a.output_path.display().to_string(),
                })
                .collect(),
        };

        serde_json::to_string_pretty(&summary)
    }
}

impl Default for BuildSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Output mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Normal output (default)
    Normal,
    /// Verbose output (show all details)
    Verbose,
    /// Quiet output (errors only)
    Quiet,
    /// JSON output (for tooling)
    Json,
}

#[allow(clippy::derivable_impls)]
impl Default for OutputMode {
    fn default() -> Self {
        Self::Normal
    }
}

/// Error formatter
pub struct ErrorFormatter {
    mode: OutputMode,
}

impl ErrorFormatter {
    /// Create new error formatter
    pub fn new(mode: OutputMode) -> Self {
        Self { mode }
    }

    /// Format compilation error
    pub fn format_error(&self, error: &str) -> String {
        match self.mode {
            OutputMode::Normal | OutputMode::Verbose => {
                format!("\x1b[31merror:\x1b[0m {}", error)
            }
            OutputMode::Quiet | OutputMode::Json => error.to_string(),
        }
    }

    /// Format warning
    pub fn format_warning(&self, warning: &str) -> String {
        match self.mode {
            OutputMode::Normal | OutputMode::Verbose => {
                format!("\x1b[33mwarning:\x1b[0m {}", warning)
            }
            OutputMode::Quiet | OutputMode::Json => warning.to_string(),
        }
    }

    /// Format success message
    pub fn format_success(&self, message: &str) -> String {
        match self.mode {
            OutputMode::Normal | OutputMode::Verbose => {
                format!("\x1b[32m{}\x1b[0m", message)
            }
            OutputMode::Quiet | OutputMode::Json => message.to_string(),
        }
    }

    /// Format info message
    pub fn format_info(&self, message: &str) -> String {
        match self.mode {
            OutputMode::Normal | OutputMode::Verbose => {
                format!("\x1b[36m{}\x1b[0m", message)
            }
            OutputMode::Quiet | OutputMode::Json => message.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_progress_new() {
        let progress = BuildProgress::new(10, OutputMode::Normal);
        assert_eq!(progress.total_modules, 10);
        assert_eq!(progress.compiled_modules, 0);
        assert!(progress.current_module.is_none());
    }

    #[test]
    fn test_build_progress_update() {
        let mut progress = BuildProgress::new(10, OutputMode::Normal);
        progress.update("module1".to_string(), Duration::from_secs(1));
        assert_eq!(progress.compiled_modules, 1);
        assert_eq!(progress.current_module, Some("module1".to_string()));
        assert!(progress.avg_compile_time.is_some());
    }

    #[test]
    fn test_build_progress_start_module() {
        let mut progress = BuildProgress::new(10, OutputMode::Normal);
        progress.start_module("module1".to_string());
        assert_eq!(progress.current_module, Some("module1".to_string()));
    }

    #[test]
    fn test_build_progress_estimate_time() {
        let mut progress = BuildProgress::new(10, OutputMode::Normal);
        progress.update("module1".to_string(), Duration::from_secs(2));

        let eta = progress.estimate_remaining_time();
        assert!(eta.is_some());
        // 9 modules remaining * 2 seconds = 18 seconds
        assert!(eta.unwrap().as_secs() >= 18);
    }

    #[test]
    fn test_build_summary_new() {
        let summary = BuildSummary::new();
        assert_eq!(summary.module_count, 0);
        assert_eq!(summary.cache_hit_rate, 0.0);
        assert_eq!(summary.artifacts.len(), 0);
    }

    #[test]
    fn test_build_summary_from_cache_stats() {
        let stats = CacheStats {
            total_modules: 10,
            cached_modules: 8,
            recompiled_modules: 2,
            cache_hit_rate: 0.8,
            cache_size_bytes: 1024,
            cache_entries: 10,
            time_saved: Duration::from_secs(5),
        };

        let summary = BuildSummary::from_cache_stats(&stats, Duration::from_secs(5));
        assert_eq!(summary.module_count, 10);
        assert_eq!(summary.cache_hit_rate, 0.8);
        assert_eq!(summary.total_time, Duration::from_secs(5));
    }

    #[test]
    fn test_build_summary_cache_hit_rate_zero_modules() {
        let stats = CacheStats {
            total_modules: 0,
            cached_modules: 0,
            recompiled_modules: 0,
            cache_hit_rate: 0.0,
            cache_size_bytes: 0,
            cache_entries: 0,
            time_saved: Duration::ZERO,
        };

        let summary = BuildSummary::from_cache_stats(&stats, Duration::from_secs(0));
        assert_eq!(summary.cache_hit_rate, 0.0);
    }

    #[test]
    fn test_build_summary_to_json() {
        let mut summary = BuildSummary::new();
        summary.total_time = Duration::from_secs(5);
        summary.module_count = 10;
        summary.cache_hit_rate = 0.8;

        let json = summary.to_json().unwrap();
        assert!(json.contains("\"success\": true"));
        assert!(json.contains("\"modules\": 10"));
    }

    #[test]
    fn test_output_mode_default() {
        assert_eq!(OutputMode::default(), OutputMode::Normal);
    }

    #[test]
    fn test_error_formatter_error() {
        let formatter = ErrorFormatter::new(OutputMode::Normal);
        let error = formatter.format_error("test error");
        assert!(error.contains("error:"));
        assert!(error.contains("test error"));
    }

    #[test]
    fn test_error_formatter_warning() {
        let formatter = ErrorFormatter::new(OutputMode::Normal);
        let warning = formatter.format_warning("test warning");
        assert!(warning.contains("warning:"));
        assert!(warning.contains("test warning"));
    }

    #[test]
    fn test_error_formatter_success() {
        let formatter = ErrorFormatter::new(OutputMode::Normal);
        let success = formatter.format_success("build succeeded");
        assert!(success.contains("build succeeded"));
    }

    #[test]
    fn test_error_formatter_info() {
        let formatter = ErrorFormatter::new(OutputMode::Normal);
        let info = formatter.format_info("info message");
        assert!(info.contains("info message"));
    }

    #[test]
    fn test_error_formatter_quiet_mode() {
        let formatter = ErrorFormatter::new(OutputMode::Quiet);
        let error = formatter.format_error("test error");
        // Quiet mode should not add color codes
        assert!(!error.contains("\x1b"));
        assert_eq!(error, "test error");
    }
}
