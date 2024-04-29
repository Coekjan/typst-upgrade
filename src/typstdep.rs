use std::{fmt::Display, str::FromStr};

use regex::Regex;
use semver::Version;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypstDep {
    namespace: String,
    name: String,
    version: Version,
}

impl Display for TypstDep {
    /// Format a [`TypstDep`] into a [`String`] ("@namespace/name:version")
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}/{}:{}", self.namespace, self.name, self.version)
    }
}

impl FromStr for TypstDep {
    type Err = &'static str;

    /// Parse a [`&str`] ("@namespace/name:version") into a [`TypstDep`]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cap = Regex::new(Self::FORMAT)
            .unwrap()
            .captures(s)
            .ok_or("Invalid format")?;

        Ok(TypstDep {
            namespace: cap["namespace"].to_string(),
            name: cap["name"].to_string(),
            version: Version::parse(&cap["version"]).map_err(|_| "Invalid version")?,
        })
    }
}

impl TypstDep {
    const FORMAT: &'static str =
        r"@(?P<namespace>[\w-]+)/(?P<name>[\w-]+):(?P<version>\d+\.\d+\.\d+)";

    pub fn is_local(&self) -> bool {
        self.namespace == "local"
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> Version {
        self.version.clone()
    }

    pub fn upgrade_to(&self, version: Version) -> TypstDep {
        TypstDep {
            namespace: self.namespace.clone(),
            name: self.name.clone(),
            version,
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::TypstDep;

    #[test]
    fn parse() {
        let package = "@preview/package:1.2.3";

        let dep = TypstDep::from_str(package);
        assert!(dep.is_ok());

        let dep = dep.unwrap();
        assert_eq!(dep.to_string(), package);
        assert_eq!(dep.namespace(), "preview");
        assert_eq!(dep.name(), "package");
        assert_eq!(dep.version().to_string(), "1.2.3");
    }

    #[test]
    fn upgrade() {
        let package = "@preview/package:1.2.3";

        let dep = TypstDep::from_str(package).unwrap();
        let new = dep.upgrade_to("2.0.0".parse().unwrap());

        assert_eq!(new.version().to_string(), "2.0.0");
        assert_eq!(new.to_string(), "@preview/package:2.0.0");
    }
}
