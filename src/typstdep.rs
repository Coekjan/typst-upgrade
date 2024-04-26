use std::{collections::HashMap, fmt::Display, str::FromStr};

use once_cell::sync::Lazy;
use regex::Regex;
use semver::Version;

static TYPST_PACKAGE_META: Lazy<HashMap<String, Vec<Version>>> = Lazy::new(|| {
    let raw_meta = reqwest::blocking::get("https://packages.typst.org/preview/index.json")
        .expect("Failed to fetch package metadata")
        .json::<serde_json::Value>()
        .expect("Failed to parse package metadata")
        .as_array()
        .expect("Invalid package metadata")
        .iter()
        .map(|v| {
            let package = v.as_object().expect("Invalid package metadata");
            let name = package
                .get("name")
                .expect("Package name not found")
                .as_str()
                .unwrap();
            let version = Version::parse(
                package
                    .get("version")
                    .expect("Package version not found")
                    .as_str()
                    .unwrap(),
            )
            .unwrap();
            (name.to_string(), version)
        })
        .collect::<Vec<_>>();

    let mut result = HashMap::new();
    for (name, version) in raw_meta {
        result.entry(name).or_insert_with(Vec::new).push(version);
    }

    result
});

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypstDep {
    namespace: String,
    name: String,
    version: Version,
}

pub struct TypstDepUpgrader {
    dep: TypstDep,
    ver: Vec<TypstDep>,
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

    pub fn upgrade(&self) -> TypstDepUpgrader {
        if self.is_local() {
            eprintln!("Local package {self} is not upgradable");

            return TypstDepUpgrader {
                dep: self.clone(),
                ver: Vec::new(),
            };
        }

        if self.namespace != "preview" {
            panic!("Unknown namespace {} for package {}", self.namespace, self);
        }

        let ver: Vec<_> = TYPST_PACKAGE_META
            .get(&self.name)
            .expect("Package not found")
            .iter()
            .filter(|&version| *version > self.version)
            .map(|version| TypstDep {
                namespace: self.namespace.clone(),
                name: self.name.clone(),
                version: version.clone(),
            })
            .collect();

        TypstDepUpgrader {
            dep: self.clone(),
            ver,
        }
    }
}

impl Display for TypstDepUpgrader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> [", self.dep)?;
        for (i, ver) in self.ver.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, " {}", ver.version)?;
        }
        write!(f, " ]")
    }
}

impl TypstDepUpgrader {
    pub fn next(&self, compatible: bool) -> Option<TypstDep> {
        self.ver
            .iter()
            .filter(|dep| {
                if compatible {
                    dep.version.major == self.dep.version.major
                } else {
                    true
                }
            })
            .max_by_key(|dep| dep.version.clone())
            .cloned()
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::{TypstDep, TypstDepUpgrader};

    #[test]
    fn parse() {
        let package = "@preview/package:1.2.3";

        let dep = TypstDep::from_str(package);
        assert!(dep.is_ok());

        let dep = dep.unwrap();
        assert_eq!(format!("{}", dep), package);
        assert_eq!(dep.namespace, "preview");
        assert_eq!(dep.name, "package");
        assert_eq!(dep.version.to_string(), "1.2.3");
    }

    #[test]
    fn upgrade() {
        let dep = TypstDep::from_str("@preview/package:1.2.3").unwrap();

        let upgrader = TypstDepUpgrader {
            dep: dep.clone(),
            ver: Vec::new(),
        };
        assert!(upgrader.next(true).is_none());
        assert!(upgrader.next(false).is_none());

        let upgrader = TypstDepUpgrader {
            dep: dep.clone(),
            ver: vec![TypstDep::from_str("@preview/package:2.0.0").unwrap()],
        };
        assert!(upgrader.next(true).is_none());

        let next_incompat = upgrader.next(false);
        assert!(next_incompat.is_some());

        let next_incompat = next_incompat.unwrap();
        assert_eq!(format!("{}", next_incompat), "@preview/package:2.0.0");

        let upgrader = TypstDepUpgrader {
            dep,
            ver: vec![
                TypstDep::from_str("@preview/package:1.2.4").unwrap(),
                TypstDep::from_str("@preview/package:1.3.0").unwrap(),
                TypstDep::from_str("@preview/package:2.0.0").unwrap(),
            ],
        };

        let next_compat = upgrader.next(true);
        assert!(next_compat.is_some());

        let next_compat = next_compat.unwrap();
        assert_eq!(format!("{}", next_compat), "@preview/package:1.3.0");

        let next_incompat = upgrader.next(false);
        assert!(next_incompat.is_some());

        let next_incompat = next_incompat.unwrap();
        assert_eq!(format!("{}", next_incompat), "@preview/package:2.0.0");
    }
}
