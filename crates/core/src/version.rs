//! Version representation and calculation.

use crate::config::{MajorMinor, VersionPart};
use serde::Serialize;

/// Semantic version representation used by MinVer.
///
/// # Examples
/// Parsing and formatting a version:
/// ```rust
/// use minver_rs::{Version, MinVerError};
///
/// let version: Version = "1.2.3".parse()?;
/// assert_eq!(version.to_string(), "1.2.3");
/// # Ok::<_, MinVerError>(())
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Vec<String>,
    pub build_metadata: Option<String>,
}

impl Version {
    /// Create a new version.
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: Vec::new(),
            build_metadata: None,
        }
    }

    /// Create a version from semver.
    pub fn from_semver(semver: &semver::Version) -> Self {
        Self {
            major: semver.major as u32,
            minor: semver.minor as u32,
            patch: semver.patch as u32,
            prerelease: vec![],
            build_metadata: None,
        }
    }

    /// Create a version from semver, preserving all fields including prerelease and build metadata.
    pub fn from_semver_full(semver: &semver::Version) -> Self {
        let prerelease: Vec<String> = semver
            .pre
            .as_str()
            .split('.')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let build_metadata = if semver.build.is_empty() {
            None
        } else {
            Some(semver.build.as_str().to_string())
        };

        Self {
            major: semver.major as u32,
            minor: semver.minor as u32,
            patch: semver.patch as u32,
            prerelease,
            build_metadata,
        }
    }

    /// Convert to semver.
    pub fn to_semver(&self) -> semver::Version {
        semver::Version::new(self.major as u64, self.minor as u64, self.patch as u64)
    }

    /// Check if this version is a pre-release (has prerelease identifiers).
    pub fn is_prerelease(&self) -> bool {
        !self.prerelease.is_empty()
    }

    /// Check if this version is RTM (Release To Manufacturing) - i.e., NOT a pre-release.
    pub fn is_rtm(&self) -> bool {
        self.prerelease.is_empty()
    }

    /// Apply auto-increment to create the next version after an RTM release.
    ///
    /// For RTM versions with height > 0:
    /// - Major: 2.0.0 -> 3.0.0
    /// - Minor: 2.3.0 -> 2.4.0
    /// - Patch: 2.3.4 -> 2.3.5
    pub fn increment(&self, part: &VersionPart) -> Self {
        match part {
            VersionPart::Major => Self {
                major: self.major + 1,
                minor: 0,
                patch: 0,
                prerelease: Vec::new(),
                build_metadata: None,
            },
            VersionPart::Minor => Self {
                major: self.major,
                minor: self.minor + 1,
                patch: 0,
                prerelease: Vec::new(),
                build_metadata: None,
            },
            VersionPart::Patch => Self {
                major: self.major,
                minor: self.minor,
                patch: self.patch + 1,
                prerelease: Vec::new(),
                build_metadata: None,
            },
        }
    }

    /// Check if this version satisfies the minimum major.minor constraint.
    pub fn satisfies(&self, min_major_minor: &MajorMinor, _default_prerelease: &[String]) -> Self {
        if self.major > min_major_minor.major
            || (self.major == min_major_minor.major && self.minor >= min_major_minor.minor)
        {
            self.clone()
        } else {
            Version::new(min_major_minor.major, min_major_minor.minor, 0)
        }
    }

    /// Add prerelease identifiers and height for a post-RTM version.
    ///
    /// Used when: We're past an RTM tag by `height` commits.
    /// Result: {incremented_version}-{default_prerelease}.{height}
    /// Example: 1.2.3 + Patch + height=5 + "alpha.0" -> 1.2.4-alpha.0.5
    pub fn with_rtm_height(
        &self,
        height: u32,
        auto_increment: &VersionPart,
        default_prerelease: &[String],
    ) -> Self {
        if height == 0 {
            return self.clone();
        }

        let incremented = self.increment(auto_increment);
        let mut prerelease = default_prerelease.to_vec();
        prerelease.push(height.to_string());

        Self {
            major: incremented.major,
            minor: incremented.minor,
            patch: incremented.patch,
            prerelease,
            build_metadata: None,
        }
    }

    /// Append height to existing prerelease identifiers.
    ///
    /// Used when: We're past a pre-release tag by `height` commits.
    /// Result: {version}-{existing_prerelease}.{height}
    /// Example: 1.0.0-beta.1 + height=3 -> 1.0.0-beta.1.3
    pub fn with_prerelease_height(&self, height: u32) -> Self {
        if height == 0 {
            return self.clone();
        }

        let mut prerelease = self.prerelease.clone();
        prerelease.push(height.to_string());

        Self {
            major: self.major,
            minor: self.minor,
            patch: self.patch,
            prerelease,
            build_metadata: None, // Build metadata from tag is NOT carried over
        }
    }

    /// Apply minimum major.minor constraint.
    ///
    /// If current version is below the minimum, return minimum with default prerelease.
    pub fn apply_minimum(&self, minimum: &MajorMinor, default_prerelease: &[String]) -> Self {
        // If version is already >= minimum, return as-is
        if self.major > minimum.major {
            return self.clone();
        }
        if self.major == minimum.major && self.minor >= minimum.minor {
            return self.clone();
        }

        // Otherwise, return minimum with default prerelease
        Self {
            major: minimum.major,
            minor: minimum.minor,
            patch: 0,
            prerelease: default_prerelease.to_vec(),
            build_metadata: None,
        }
    }

    /// Merge build metadata from tag and config.
    ///
    /// Rules:
    /// - If only tag has metadata: use tag's
    /// - If only config has metadata: use config's
    /// - If both: join with "." (tag.config)
    pub fn with_merged_build_metadata(
        &self,
        tag_metadata: Option<&str>,
        config_metadata: Option<&str>,
    ) -> Self {
        let merged = match (tag_metadata, config_metadata) {
            (None, None) => None,
            (Some(t), None) => Some(t.to_string()),
            (None, Some(c)) => Some(c.to_string()),
            (Some(t), Some(c)) => Some(format!("{}.{}", t, c)),
        };

        Self {
            major: self.major,
            minor: self.minor,
            patch: self.patch,
            prerelease: self.prerelease.clone(),
            build_metadata: merged,
        }
    }

    /// Create version with height.
    pub fn with_height(
        &self,
        height: u32,
        _auto_increment: &VersionPart,
        default_prerelease: &[String],
    ) -> Self {
        let mut version = self.clone();
        version.patch += height;
        if height > 0 {
            version.prerelease = default_prerelease.to_vec();
        }
        version
    }

    /// Create version with build metadata.
    pub fn with_build_metadata(&self, build_metadata: &str) -> Self {
        let mut version = self.clone();
        version.build_metadata = Some(build_metadata.to_string());
        version
    }

    /// Create default version.
    pub fn default(default_prerelease: &[String]) -> Self {
        Self {
            major: 0,
            minor: 0,
            patch: 0,
            prerelease: default_prerelease.to_vec(),
            build_metadata: None,
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if !self.prerelease.is_empty() {
            write!(f, "-{}", self.prerelease.join("."))?;
        }

        if let Some(ref build) = self.build_metadata {
            write!(f, "+{}", build)?;
        }

        Ok(())
    }
}

impl std::str::FromStr for Version {
    type Err = crate::error::MinVerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let semver = semver::Version::parse(s)
            .map_err(|e| crate::error::MinVerError::InvalidSemver(e.to_string()))?;
        Ok(Version::from_semver(&semver))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.prerelease == other.prerelease
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Simple version comparison
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }
}

impl MajorMinor {
    /// Create a new major.minor constraint.
    pub fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_display() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
    }
}
