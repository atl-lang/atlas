//! Runtime configuration for execution limits and sandboxing
//!
//! Provides configuration options for controlling Atlas runtime behavior,
//! including execution limits, memory constraints, and capability restrictions.

use crate::stdlib::{stdout_writer, OutputWriter};
use std::time::Duration;

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
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::RuntimeConfig;
    /// use atlas_runtime::stdlib::{OutputWriter, stdout_writer};
    /// use std::sync::{Arc, Mutex};
    ///
    /// // Capture output in a buffer
    /// let buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    /// // (wrap buf in a Write newtype, then pass as OutputWriter)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_permissive() {
        let config = RuntimeConfig::default();
        assert!(config.max_execution_time.is_none());
        assert!(config.max_memory_bytes.is_none());
        assert!(config.allow_io);
        assert!(config.allow_network);
    }

    #[test]
    fn test_sandboxed_config_is_restrictive() {
        let config = RuntimeConfig::sandboxed();
        assert_eq!(config.max_execution_time, Some(Duration::from_secs(5)));
        assert_eq!(config.max_memory_bytes, Some(10_000_000));
        assert!(!config.allow_io);
        assert!(!config.allow_network);
    }

    #[test]
    fn test_fluent_api() {
        let config = RuntimeConfig::new()
            .with_max_execution_time(Duration::from_secs(10))
            .with_max_memory_bytes(50_000_000)
            .with_io_allowed(false)
            .with_network_allowed(true);

        assert_eq!(config.max_execution_time, Some(Duration::from_secs(10)));
        assert_eq!(config.max_memory_bytes, Some(50_000_000));
        assert!(!config.allow_io);
        assert!(config.allow_network);
    }
}
