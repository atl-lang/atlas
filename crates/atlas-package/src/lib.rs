//! Atlas Package Management (phase-07 + phase-08a + phase-08b)
//!
//! Package manifest system for atlas.toml files, dependency management,
//! lockfile generation, workspace support, and package registry.

pub mod cache;
pub mod downloader;
pub mod lockfile;
pub mod manifest;
pub mod registry;
pub mod resolver;
pub mod validator;

pub use cache::PackageCache;
pub use downloader::Downloader;
pub use lockfile::{LockedPackage, LockedSource, Lockfile};
pub use manifest::{
    Dependency, DependencySource, Feature, PackageManifest, VersionConstraint, Workspace,
};
pub use registry::{
    LocalRegistry, PackageMetadata, Registry, RegistryError, RegistryManager, RegistryResult,
    RemoteRegistry,
};
pub use resolver::{
    DependencyGraph, Resolution, ResolvedPackage, Resolver, ResolverError, ResolverResult,
    VersionSolver,
};
pub use validator::{ValidationError, Validator};

/// Package management errors
#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("Failed to parse manifest: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Failed to serialize manifest: {0}")]
    SerializeError(#[from] toml::ser::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Semver error: {0}")]
    SemverError(#[from] semver::Error),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid field value: {field} - {reason}")]
    InvalidField { field: String, reason: String },
}

pub type Result<T> = std::result::Result<T, PackageError>;
