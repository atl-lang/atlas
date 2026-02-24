//! Hotspot detection
//!
//! Analyses profiler data to find instruction locations that account for
//! a disproportionate share of total execution.

use crate::bytecode::Opcode;
use crate::profiler::collector::ProfileCollector;

/// A single hotspot — an instruction location above the detection threshold
#[derive(Debug, Clone)]
pub struct Hotspot {
    /// Instruction pointer of the hot location
    pub ip: usize,
    /// Times this location was executed
    pub count: u64,
    /// Percentage of total instructions (0–100)
    pub percentage: f64,
    /// The opcode at this location (None if unknown)
    pub opcode: Option<Opcode>,
}

/// A hot opcode summary
#[derive(Debug, Clone)]
pub struct HotOpcode {
    pub opcode: Opcode,
    pub count: u64,
    pub percentage: f64,
}

/// Detects hotspots in profiler data
#[derive(Debug, Clone)]
pub struct HotspotDetector {
    /// Minimum percentage to qualify as a hotspot (default 1.0 = 1%)
    threshold_pct: f64,
    /// Maximum number of hotspots to return
    max_hotspots: usize,
}

impl HotspotDetector {
    /// Create a detector with default settings (1% threshold, 20 hotspots)
    pub fn new() -> Self {
        Self {
            threshold_pct: 1.0,
            max_hotspots: 20,
        }
    }

    /// Create a detector with a custom threshold percentage (0–100)
    pub fn with_threshold(threshold_pct: f64) -> Self {
        Self {
            threshold_pct: threshold_pct.clamp(0.0, 100.0),
            max_hotspots: 20,
        }
    }

    /// Set the maximum number of hotspots to return
    pub fn with_max_hotspots(mut self, n: usize) -> Self {
        self.max_hotspots = n;
        self
    }

    /// Get the threshold in use
    pub fn threshold(&self) -> f64 {
        self.threshold_pct
    }

    /// Detect hotspot locations from collector data
    ///
    /// Returns hotspots sorted by execution count (highest first),
    /// limited to those above `threshold_pct` of total instructions.
    pub fn detect(&self, collector: &ProfileCollector) -> Vec<Hotspot> {
        let total = collector.total_instructions();
        if total == 0 {
            return Vec::new();
        }

        let mut hotspots: Vec<Hotspot> = collector
            .location_counts()
            .iter()
            .filter_map(|(&ip, &count)| {
                let pct = (count as f64 / total as f64) * 100.0;
                if pct >= self.threshold_pct {
                    Some(Hotspot {
                        ip,
                        count,
                        percentage: pct,
                        opcode: collector.opcode_at(ip),
                    })
                } else {
                    None
                }
            })
            .collect();

        hotspots.sort_by(|a, b| b.count.cmp(&a.count));
        hotspots.truncate(self.max_hotspots);
        hotspots
    }

    /// Identify the top N opcodes by execution frequency
    pub fn top_opcodes(&self, collector: &ProfileCollector, n: usize) -> Vec<HotOpcode> {
        let total = collector.total_instructions();
        if total == 0 {
            return Vec::new();
        }

        collector
            .top_opcodes(n)
            .into_iter()
            .map(|(opcode, count)| HotOpcode {
                opcode,
                count,
                percentage: (count as f64 / total as f64) * 100.0,
            })
            .collect()
    }

    /// Check whether a specific location qualifies as a hotspot
    pub fn is_hotspot(&self, collector: &ProfileCollector, ip: usize) -> bool {
        let total = collector.total_instructions();
        if total == 0 {
            return false;
        }
        let count = collector.location_counts().get(&ip).copied().unwrap_or(0);
        (count as f64 / total as f64) * 100.0 >= self.threshold_pct
    }
}

impl Default for HotspotDetector {
    fn default() -> Self {
        Self::new()
    }
}
