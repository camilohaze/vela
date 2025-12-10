//! Version Constraint System
//!
//! This module handles parsing and evaluation of version constraints
//! using semantic versioning (SemVer) rules.

use crate::common::Error;
use semver::{Version, VersionReq};
use std::str::FromStr;

/// A version constraint that specifies acceptable version ranges
#[derive(Debug, Clone, PartialEq)]
pub enum VersionConstraint {
    /// Exact version match (e.g., "1.2.3")
    Exact(Version),
    /// Caret range - compatible with major version (e.g., "^1.2.3" allows 1.x.x)
    Caret(Version),
    /// Tilde range - compatible with minor version (e.g., "~1.2.3" allows 1.2.x)
    Tilde(Version),
    /// Greater than (e.g., ">1.2.3")
    GreaterThan(Version),
    /// Greater than or equal (e.g., ">=1.2.3")
    GreaterEqual(Version),
    /// Less than (e.g., "<1.2.3")
    LessThan(Version),
    /// Less than or equal (e.g., "<=1.2.3")
    LessEqual(Version),
    /// Version range (e.g., "1.0.0 - 2.0.0")
    Range(Version, Version),
    /// Wildcard - any version in major (e.g., "1.x" or "*")
    Wildcard(u64), // major version, 0 means any
}

impl VersionConstraint {
    /// Parse a version constraint from string
    pub fn parse(input: &str) -> Result<Self, Error> {
        let input = input.trim();

        match input {
            "*" => Ok(VersionConstraint::Wildcard(0)),
            s if s.starts_with("^") => {
                let version = Version::parse(&s[1..])
                    .map_err(|_| Error::InvalidVersionConstraint(s.to_string()))?;
                Ok(VersionConstraint::Caret(version))
            }
            s if s.starts_with("~") => {
                let version = Version::parse(&s[1..])
                    .map_err(|_| Error::InvalidVersionConstraint(s.to_string()))?;
                Ok(VersionConstraint::Tilde(version))
            }
            s if s.starts_with(">=") => {
                let version = Version::parse(&s[2..])
                    .map_err(|_| Error::InvalidVersionConstraint(s.to_string()))?;
                Ok(VersionConstraint::GreaterEqual(version))
            }
            s if s.starts_with(">") => {
                let version = Version::parse(&s[1..])
                    .map_err(|_| Error::InvalidVersionConstraint(s.to_string()))?;
                Ok(VersionConstraint::GreaterThan(version))
            }
            s if s.starts_with("<=") => {
                let version = Version::parse(&s[2..])
                    .map_err(|_| Error::InvalidVersionConstraint(s.to_string()))?;
                Ok(VersionConstraint::LessEqual(version))
            }
            s if s.starts_with("<") => {
                let version = Version::parse(&s[1..])
                    .map_err(|_| Error::InvalidVersionConstraint(s.to_string()))?;
                Ok(VersionConstraint::LessThan(version))
            }
            s if s.contains(" - ") => {
                let parts: Vec<&str> = s.split(" - ").collect();
                if parts.len() == 2 {
                    let min = Version::parse(parts[0])
                        .map_err(|_| Error::InvalidVersionConstraint(s.to_string()))?;
                    let max = Version::parse(parts[1])
                        .map_err(|_| Error::InvalidVersionConstraint(s.to_string()))?;
                    Ok(VersionConstraint::Range(min, max))
                } else {
                    Err(Error::InvalidVersionConstraint(s.to_string()))
                }
            }
            s if s.contains(".x") => {
                // Handle "1.x" or "1.2.x" patterns
                let parts: Vec<&str> = s.split('.').collect();
                if parts.len() >= 2 && parts[1] == "x" {
                    let major = parts[0].parse::<u64>()
                        .map_err(|_| Error::InvalidVersionConstraint(s.to_string()))?;
                    Ok(VersionConstraint::Wildcard(major))
                } else {
                    Err(Error::InvalidVersionConstraint(s.to_string()))
                }
            }
            _ => {
                // Try to parse as exact version
                let version = Version::parse(input)
                    .map_err(|_| Error::InvalidVersionConstraint(input.to_string()))?;
                Ok(VersionConstraint::Exact(version))
            }
        }
    }

    /// Check if a version satisfies this constraint
    pub fn satisfies(&self, version: &Version) -> bool {
        match self {
            VersionConstraint::Exact(req) => version == req,
            VersionConstraint::Caret(req) => {
                // ^1.2.3 allows changes in patch and minor, but not major
                version.major == req.major &&
                (version.minor >= req.minor || version.major > req.major)
            }
            VersionConstraint::Tilde(req) => {
                // ~1.2.3 allows changes in patch, but not minor or major
                version.major == req.major && version.minor == req.minor
            }
            VersionConstraint::GreaterThan(req) => version > req,
            VersionConstraint::GreaterEqual(req) => version >= req,
            VersionConstraint::LessThan(req) => version < req,
            VersionConstraint::LessEqual(req) => version <= req,
            VersionConstraint::Range(min, max) => version >= min && version <= max,
            VersionConstraint::Wildcard(major) => {
                if *major == 0 {
                    true // * matches any version
                } else {
                    version.major == *major
                }
            }
        }
    }

    /// Get all versions that could potentially satisfy this constraint
    /// This is used for optimization in the resolution algorithm
    pub fn potential_versions(&self) -> Vec<Version> {
        // This would query the registry for available versions
        // For now, return empty vec - will be implemented with registry integration
        Vec::new()
    }
}

impl std::fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionConstraint::Exact(v) => write!(f, "{}", v),
            VersionConstraint::Caret(v) => write!(f, "^{}", v),
            VersionConstraint::Tilde(v) => write!(f, "~{}", v),
            VersionConstraint::GreaterThan(v) => write!(f, ">{}", v),
            VersionConstraint::GreaterEqual(v) => write!(f, ">={}", v),
            VersionConstraint::LessThan(v) => write!(f, "<{}", v),
            VersionConstraint::LessEqual(v) => write!(f, "<={}", v),
            VersionConstraint::Range(min, max) => write!(f, "{} - {}", min, max),
            VersionConstraint::Wildcard(0) => write!(f, "*"),
            VersionConstraint::Wildcard(major) => write!(f, "{}.x", major),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_exact_version() {
        let constraint = VersionConstraint::parse("1.2.3").unwrap();
        match constraint {
            VersionConstraint::Exact(v) => assert_eq!(v.to_string(), "1.2.3"),
            _ => panic!("Expected Exact"),
        }
    }

    #[test]
    fn test_parse_caret_range() {
        let constraint = VersionConstraint::parse("^1.2.3").unwrap();
        match constraint {
            VersionConstraint::Caret(v) => assert_eq!(v.to_string(), "1.2.3"),
            _ => panic!("Expected Caret"),
        }
    }

    #[test]
    fn test_parse_tilde_range() {
        let constraint = VersionConstraint::parse("~1.2.3").unwrap();
        match constraint {
            VersionConstraint::Tilde(v) => assert_eq!(v.to_string(), "1.2.3"),
            _ => panic!("Expected Tilde"),
        }
    }

    #[test]
    fn test_parse_wildcard() {
        let constraint = VersionConstraint::parse("*").unwrap();
        match constraint {
            VersionConstraint::Wildcard(0) => {},
            _ => panic!("Expected Wildcard"),
        }
    }

    #[test]
    fn test_satisfies_exact() {
        let constraint = VersionConstraint::parse("1.2.3").unwrap();
        let version = Version::parse("1.2.3").unwrap();
        assert!(constraint.satisfies(&version));

        let other_version = Version::parse("1.2.4").unwrap();
        assert!(!constraint.satisfies(&other_version));
    }

    #[test]
    fn test_satisfies_caret() {
        let constraint = VersionConstraint::parse("^1.2.3").unwrap();
        let compatible = Version::parse("1.3.0").unwrap();
        let incompatible = Version::parse("2.0.0").unwrap();

        assert!(constraint.satisfies(&compatible));
        assert!(!constraint.satisfies(&incompatible));
    }

    #[test]
    fn test_display() {
        let constraint = VersionConstraint::parse("^1.2.3").unwrap();
        assert_eq!(constraint.to_string(), "^1.2.3");
    }
}