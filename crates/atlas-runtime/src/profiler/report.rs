//! Profile report generation
//!
//! Formats collected profiling data into human-readable performance reports.

use crate::profiler::hotspots::Hotspot;

/// Comprehensive profile report
#[derive(Debug, Clone)]
pub struct ProfileReport {
    /// Total instructions executed
    pub total_instructions: u64,
    /// Wall-clock time in seconds (None if timing was not recorded)
    pub elapsed_secs: Option<f64>,
    /// Instructions per second (None if timing was not available)
    pub ips: Option<f64>,
    /// Maximum call frame stack depth
    pub max_stack_depth: usize,
    /// Maximum value stack depth
    pub max_value_stack_depth: usize,
    /// Total named function calls
    pub function_calls: u64,
    /// Top opcodes: (name, count, percentage)
    pub top_opcodes: Vec<(String, u64, f64)>,
    /// Hotspot locations above the threshold
    pub hotspots: Vec<Hotspot>,
}

impl ProfileReport {
    /// One-line summary: instructions + time
    pub fn format_summary(&self) -> String {
        let mut s = format!("Profile: {} instructions executed", self.total_instructions);
        if let Some(elapsed) = self.elapsed_secs {
            s.push_str(&format!(", {:.3}s", elapsed));
        }
        if let Some(ips) = self.ips {
            s.push_str(&format!(" ({:.0} IPS)", ips));
        }
        s
    }

    /// Full multi-section report
    pub fn format_detailed(&self) -> String {
        let mut out = String::new();

        out.push_str("=== Atlas VM Profile Report ===\n\n");

        // --- Execution summary ---
        out.push_str("[ Execution Summary ]\n");
        out.push_str(&format!(
            "  Total instructions : {}\n",
            self.total_instructions
        ));
        if let Some(elapsed) = self.elapsed_secs {
            out.push_str(&format!("  Elapsed time       : {:.6}s\n", elapsed));
        }
        if let Some(ips) = self.ips {
            out.push_str(&format!(
                "  Throughput         : {:.0} instructions/sec\n",
                ips
            ));
        }
        out.push_str(&format!(
            "  Max frame depth    : {}\n",
            self.max_stack_depth
        ));
        out.push_str(&format!(
            "  Max value stack    : {}\n",
            self.max_value_stack_depth
        ));
        out.push_str(&format!("  Function calls     : {}\n", self.function_calls));
        out.push('\n');

        // --- Top opcodes ---
        if !self.top_opcodes.is_empty() {
            out.push_str("[ Top Opcodes ]\n");
            out.push_str(&format!(
                "  {:<24} {:>12}  {:>7}\n",
                "Opcode", "Count", "Pct"
            ));
            out.push_str("  ");
            out.push_str(&"-".repeat(47));
            out.push('\n');
            for (name, count, pct) in &self.top_opcodes {
                out.push_str(&format!("  {:<24} {:>12}  {:>6.2}%\n", name, count, pct));
            }
            out.push('\n');
        }

        // --- Hotspots ---
        if !self.hotspots.is_empty() {
            out.push_str("[ Hotspots (>= threshold) ]\n");
            out.push_str(&format!(
                "  {:<8}  {:<20} {:>12}  {:>7}\n",
                "IP", "Opcode", "Count", "Pct"
            ));
            out.push_str("  ");
            out.push_str(&"-".repeat(55));
            out.push('\n');
            for h in &self.hotspots {
                let opcode_name = h
                    .opcode
                    .map(|op| format!("{:?}", op))
                    .unwrap_or_else(|| "Unknown".to_string());
                out.push_str(&format!(
                    "  {:<8}  {:<20} {:>12}  {:>6.2}%\n",
                    h.ip, opcode_name, h.count, h.percentage
                ));
            }
            out.push('\n');
        } else {
            out.push_str("[ Hotspots ]\n  No hotspots detected above threshold.\n\n");
        }

        out
    }

    /// Opcode breakdown table only
    pub fn format_opcode_table(&self) -> String {
        if self.top_opcodes.is_empty() {
            return "No opcodes recorded.\n".to_string();
        }

        let mut out = String::new();
        out.push_str(&format!("{:<24} {:>12}  {:>7}\n", "Opcode", "Count", "Pct"));
        out.push_str(&"-".repeat(47));
        out.push('\n');
        for (name, count, pct) in &self.top_opcodes {
            out.push_str(&format!("{:<24} {:>12}  {:>6.2}%\n", name, count, pct));
        }
        out
    }
}
