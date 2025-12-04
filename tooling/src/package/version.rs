/*!
Semantic versioning utilities
*/

pub use semver::{Version as SemVersion, VersionReq};

/// Version alias
pub type Version = SemVersion;

/// Parse version string
pub fn parse_version(s: &str) -> Result<Version, semver::Error> {
    s.parse()
}

/// Parse version requirement
pub fn parse_version_req(s: &str) -> Result<VersionReq, semver::Error> {
    s.parse()
}

/// Check if version satisfies requirement
pub fn matches(version: &Version, req: &VersionReq) -> bool {
    req.matches(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let v = parse_version("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_parse_version_req() {
        let req = parse_version_req("^1.0").unwrap();
        assert!(matches(&parse_version("1.5.0").unwrap(), &req));
        assert!(!matches(&parse_version("2.0.0").unwrap(), &req));
    }

    #[test]
    fn test_matches() {
        let v1 = parse_version("1.5.0").unwrap();
        let v2 = parse_version("2.0.0").unwrap();
        let req = parse_version_req("^1.0").unwrap();

        assert!(matches(&v1, &req));
        assert!(!matches(&v2, &req));
    }

    #[test]
    fn test_exact_version() {
        let v = parse_version("1.2.3").unwrap();
        let req = parse_version_req("=1.2.3").unwrap();

        assert!(matches(&v, &req));
    }

    #[test]
    fn test_range_version() {
        let v = parse_version("1.5.0").unwrap();
        let req = parse_version_req(">=1.0, <2.0").unwrap();

        assert!(matches(&v, &req));
    }
}
