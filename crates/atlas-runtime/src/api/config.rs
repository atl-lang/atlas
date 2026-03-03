//! Runtime configuration for execution limits and sandboxing
//!
//! Provides configuration options for controlling Atlas runtime behavior,
//! including execution limits, memory constraints, and capability restrictions.

use crate::stdlib::{stdout_writer, OutputWriter};
use crate::value::RuntimeError;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Execution limits for sandbox enforcement
///
/// Tracks execution time and provides methods to check if limits are exceeded.
/// Used by both interpreter and VM to enforce timeout limits.
#[derive(Debug)]
pub struct ExecutionLimits {
    /// Maximum execution time before timeout (None = unlimited)
    pub max_time: Option<Duration>,
    /// Execution start time (set when limits are activated)
    start_time: Option<Instant>,
    /// Instruction counter for amortized time checks in VM
    instruction_count: AtomicU64,
    /// Check interval: how many instructions between time checks (for VM performance)
    check_interval: u64,
}

impl ExecutionLimits {
    /// Create execution limits from configuration
    pub fn from_config(config: &RuntimeConfig) -> Self {
        Self {
            max_time: config.max_execution_time,
            start_time: None,
            instruction_count: AtomicU64::new(0),
            // Check every 10000 instructions to amortize syscall overhead
            check_interval: 10000,
        }
    }

    /// Create unlimited execution limits (no timeout)
    pub fn unlimited() -> Self {
        Self {
            max_time: None,
            start_time: None,
            instruction_count: AtomicU64::new(0),
            check_interval: 10000,
        }
    }

    /// Start the execution timer
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.instruction_count.store(0, Ordering::Relaxed);
    }

    /// Check if execution time limit has been exceeded
    ///
    /// Returns Ok(()) if within limits, Err if timeout exceeded.
    pub fn check_timeout(&self) -> Result<(), RuntimeError> {
        let Some(max_time) = self.max_time else {
            return Ok(()); // No limit configured
        };

        let Some(start) = self.start_time else {
            return Ok(()); // Timer not started
        };

        let elapsed = start.elapsed();
        if elapsed > max_time {
            return Err(RuntimeError::Timeout {
                elapsed,
                limit: max_time,
            });
        }

        Ok(())
    }

    /// Increment instruction count and check timeout if interval reached (for VM)
    ///
    /// This amortizes the cost of time checks over many instructions.
    /// Returns Ok(()) if within limits, Err if timeout exceeded.
    #[inline]
    pub fn tick_and_check(&self) -> Result<(), RuntimeError> {
        if self.max_time.is_none() {
            return Ok(()); // Fast path: no limit configured
        }

        let count = self.instruction_count.fetch_add(1, Ordering::Relaxed);
        if count.is_multiple_of(self.check_interval) {
            self.check_timeout()
        } else {
            Ok(())
        }
    }

    /// Get elapsed execution time
    pub fn elapsed(&self) -> Option<Duration> {
        self.start_time.map(|s| s.elapsed())
    }

    /// Check if limits are active (has a timeout configured)
    pub fn is_active(&self) -> bool {
        self.max_time.is_some()
    }
}

impl Clone for ExecutionLimits {
    fn clone(&self) -> Self {
        Self {
            max_time: self.max_time,
            start_time: self.start_time,
            instruction_count: AtomicU64::new(self.instruction_count.load(Ordering::Relaxed)),
            check_interval: self.check_interval,
        }
    }
}

/// Runtime configuration for execution limits and sandboxing
///
/// Controls runtime behavior including execution time limits, memory constraints,
/// and capability restrictions (IO, network access).
///
/// # Examples
///
/// ```
/// use atlas_runtime::api::RuntimeConfig;
/// use std::time::Duration;
///
/// // Default config (permissive)
/// let config = RuntimeConfig::default();
///
/// // Sandboxed config (restrictive)
/// let config = RuntimeConfig::sandboxed();
///
/// // Custom config
/// let config = RuntimeConfig::new()
///     .with_max_execution_time(Duration::from_secs(10))
///     .with_max_memory_bytes(50_000_000) // 50MB
///     .with_io_allowed(false)
///     .with_network_allowed(false);
/// ```
#[derive(Clone)]
pub struct RuntimeConfig {
    /// Maximum execution time before timeout (None = unlimited)
    pub max_execution_time: Option<Duration>,

    /// Maximum memory allocation in bytes (None = unlimited)
    pub max_memory_bytes: Option<usize>,

    /// Whether IO operations are allowed (file read/write)
    pub allow_io: bool,

    /// Whether network operations are allowed
    pub allow_network: bool,

    /// Output destination for print(). Defaults to stdout.
    pub output: OutputWriter,
}

impl std::fmt::Debug for RuntimeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeConfig")
            .field("max_execution_time", &self.max_execution_time)
            .field("max_memory_bytes", &self.max_memory_bytes)
            .field("allow_io", &self.allow_io)
            .field("allow_network", &self.allow_network)
            .field("output", &"<output writer>")
            .finish()
    }
}

impl RuntimeConfig {
    /// Create a new config with default permissive settings
    ///
    /// Default settings:
    /// - No execution time limit
    /// - No memory limit
    /// - IO allowed
    /// - Network allowed
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::RuntimeConfig;
    ///
    /// let config = RuntimeConfig::new();
    /// assert!(config.allow_io);
    /// assert!(config.allow_network);
    /// ```
    pub fn new() -> Self {
        Self {
            max_execution_time: None,
            max_memory_bytes: None,
            allow_io: true,
            allow_network: true,
            output: stdout_writer(),
        }
    }

    /// Create a sandboxed config with restrictive defaults
    ///
    /// Sandboxed settings:
    /// - 5 second execution timeout
    /// - 10MB memory limit
    /// - IO disabled
    /// - Network disabled
    ///
    /// Suitable for running untrusted code.
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::RuntimeConfig;
    ///
    /// let config = RuntimeConfig::sandboxed();
    /// assert!(!config.allow_io);
    /// assert!(!config.allow_network);
    /// assert!(config.max_execution_time.is_some());
    /// ```
    pub fn sandboxed() -> Self {
        Self {
            max_execution_time: Some(Duration::from_secs(5)),
            max_memory_bytes: Some(10_000_000), // 10MB
            allow_io: false,
            allow_network: false,
            output: stdout_writer(),
        }
    }

    /// Redirect all `print()` output to a custom writer.
    ///
    /// The default writer goes to real stdout. Pass any `Arc<Mutex<Box<dyn Write + Send>>>`
    /// to capture or redirect output — useful for testing or embedding.
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::RuntimeConfig;
    /// use atlas_runtime::stdlib::stdout_writer;
    ///
    /// // Explicitly set stdout (same as the default):
    /// let config = RuntimeConfig::new().with_output(stdout_writer());
    /// assert!(config.allow_io);
    /// ```
    pub fn with_output(mut self, output: OutputWriter) -> Self {
        self.output = output;
        self
    }

    /// Set maximum execution time
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::RuntimeConfig;
    /// use std::time::Duration;
    ///
    /// let config = RuntimeConfig::new()
    ///     .with_max_execution_time(Duration::from_secs(30));
    /// ```
    pub fn with_max_execution_time(mut self, duration: Duration) -> Self {
        self.max_execution_time = Some(duration);
        self
    }

    /// Set maximum memory allocation in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::RuntimeConfig;
    ///
    /// let config = RuntimeConfig::new()
    ///     .with_max_memory_bytes(100_000_000); // 100MB
    /// ```
    pub fn with_max_memory_bytes(mut self, bytes: usize) -> Self {
        self.max_memory_bytes = Some(bytes);
        self
    }

    /// Set whether IO operations are allowed
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::RuntimeConfig;
    ///
    /// let config = RuntimeConfig::new()
    ///     .with_io_allowed(false);
    /// ```
    pub fn with_io_allowed(mut self, allowed: bool) -> Self {
        self.allow_io = allowed;
        self
    }

    /// Set whether network operations are allowed
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::RuntimeConfig;
    ///
    /// let config = RuntimeConfig::new()
    ///     .with_network_allowed(false);
    /// ```
    pub fn with_network_allowed(mut self, allowed: bool) -> Self {
        self.allow_network = allowed;
        self
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self::new()
    }
}
