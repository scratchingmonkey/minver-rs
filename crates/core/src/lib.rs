//! # minver-rs core library
//! Minimalistic version calculation from Git tags, mirroring the original .NET MinVer behavior.
//!
//! ## Quick Start
//!
//! Get the version for the current Git repository.
//!
//! ```rust,no_run
//! # use minver_rs::MinVerError;
//! use minver_rs::{calculate_version, Config};
//!
//! // Use default configuration
//! let config = Config::default();
//!
//! // Calculate version from the current directory
//! let result = calculate_version(".", &config)?;
//!
//! println!("Calculated version: {}", result);
//! # Ok::<_, MinVerError>(())
//! ```
//!
//! ## Strict vs. fallback entry points
//! - [`calculate_version`] — requires a real Git repository and errors otherwise.
//! - [`calculate_version_with_fallback`] — returns the default version when no repository is found.

pub mod config;
pub mod error;
pub mod git;
pub mod tags;
pub mod version;

pub use config::{Config, Verbosity, VersionPart};
pub use error::{MinVerError, Result};
pub use git::Repository;
pub use version::Version;

/// Calculate the version for the given repository using the minver-rs algorithm.
///
/// # Examples
/// Returning an error when the target is not a Git repository:
/// ```rust
/// use minver_rs::{calculate_version, Config, MinVerError};
///
/// let config = Config::default();
/// let err = calculate_version("/tmp/not-a-repo-minver", &config).unwrap_err();
/// match err {
///     MinVerError::GitRepoNotFound(_) => {},
///     other => panic!("unexpected error: {other}"),
/// }
/// ```
///
/// # Errors
/// - [`MinVerError::GitRepoNotFound`] if the path is not inside a Git repository.
/// - [`MinVerError::GitCommand`] or [`MinVerError::Other`] for underlying Git failures.
/// - [`MinVerError::InvalidSemver`] if tags contain invalid SemVer.
pub fn calculate_version(
    work_dir: impl Into<std::path::PathBuf>,
    config: &Config,
) -> Result<CalculationResult> {
    let work_dir = work_dir.into();

    // Try to discover the repository
    let repo = Repository::discover(&work_dir)?;

    // Check for shallow clone warning
    if repo.is_shallow() {
        tracing::warn!("Shallow repository detected. Version calculation may be incorrect. Fetch full history with 'git fetch --unshallow'.");
    }

    // Calculate the version
    let (version, height, is_from_tag) = git::calculate_version(&repo, config)?;

    Ok(CalculationResult {
        version,
        height,
        is_from_tag,
        work_dir,
    })
}

/// Calculate the version, falling back to the default version when no repository is found.
///
/// # Examples
/// Using a non-repository directory will return the default version instead of erroring:
/// ```rust
/// use minver_rs::{calculate_version_with_fallback, Config, MinVerError};
///
/// let config = Config::default();
/// let result = calculate_version_with_fallback("/tmp/not-a-repo-minver", &config)?;
/// assert_eq!(result.to_string(), "0.0.0-alpha.0");
/// assert!(!result.is_from_tag);
/// # Ok::<_, MinVerError>(())
/// ```
///
/// # Errors
/// - [`MinVerError::GitCommand`] or [`MinVerError::Other`] for unexpected Git errors during fallback discovery.
pub fn calculate_version_with_fallback(
    work_dir: impl Into<std::path::PathBuf>,
    config: &Config,
) -> Result<CalculationResult> {
    let work_dir = work_dir.into();

    // Try to discover and calculate version
    let (version, height, is_from_tag) = git::calculate_version_fallback(&work_dir, config)?;

    Ok(CalculationResult {
        version,
        height,
        is_from_tag,
        work_dir,
    })
}

/// Result of a version calculation.
///
/// # Examples
/// ```rust
/// use minver_rs::{calculate_version_with_fallback, Config, MinVerError};
///
/// let result = calculate_version_with_fallback("/tmp/not-a-repo-minver", &Config::default())?;
/// assert_eq!(result.version.to_string(), "0.0.0-alpha.0");
/// assert_eq!(result.height, 0);
/// assert!(!result.is_from_tag);
/// # Ok::<_, MinVerError>(())
/// ```
#[derive(Debug, Clone)]
pub struct CalculationResult {
    pub version: Version,
    pub height: u32,
    pub is_from_tag: bool,
    pub work_dir: std::path::PathBuf,
}

impl CalculationResult {
    /// Returns the version as a string (alias for [`Display`](std::fmt::Display)).
    pub fn to_string(&self) -> String {
        self.version.to_string()
    }
}

impl std::fmt::Display for CalculationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.version)
    }
}
