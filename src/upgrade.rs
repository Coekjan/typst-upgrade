use std::{collections::HashMap, fmt::Display, str::FromStr};

use once_cell::sync::Lazy;
use semver::Version;
use typst_syntax::{
    ast::{AstNode, Expr, ModuleImport},
    SyntaxKind, SyntaxNode,
};

use crate::typstdep::TypstDep;

pub struct TypstNodeUpgrader<'a> {
    root: &'a SyntaxNode,
    verbose: bool,
    compatible: bool,
}

impl<'a> TypstNodeUpgrader<'a> {
    pub fn new(root: &'a SyntaxNode, verbose: bool, compatible: bool) -> Self {
        Self {
            root,
            verbose,
            compatible,
        }
    }

    pub fn convert(&self) -> SyntaxNode {
        match self.root.kind() {
            SyntaxKind::Markup | SyntaxKind::Code => self.convert_recursively(self.root),
            kind => panic!("Unexpected node kind: {:?}", kind),
        }
    }

    fn convert_recursively(&self, node: &SyntaxNode) -> SyntaxNode {
        if let Some(module_import) = node.cast::<ModuleImport>() {
            let Expr::Str(s) = module_import.source() else {
                if self.verbose {
                    eprintln!(
                        "Cannot upgrade non-string module import: {}",
                        node.clone().into_text(),
                    );
                }
                return node.clone();
            };
            let Ok(dep) = TypstDep::from_str(s.to_untyped().text()) else {
                return node.clone();
            };
            if dep.is_local() {
                if self.verbose {
                    eprintln!("Local package {dep} is not upgradable");
                }
                return node.clone();
            }
            let Some(next) = TypstDepUpgrader::build(&dep).next(self.compatible) else {
                if self.verbose {
                    eprintln!("Package {dep} is already up-to-date");
                }
                return node.clone();
            };
            SyntaxNode::inner(
                node.kind(),
                node.children()
                    .map(|child| match child.kind() {
                        SyntaxKind::Str
                            if child.text() == module_import.source().to_untyped().text() =>
                        {
                            SyntaxNode::leaf(SyntaxKind::Str, format!("\"{}\"", next))
                        }
                        _ => self.convert_recursively(child),
                    })
                    .collect(),
            )
        } else if node.children().len() == 0 {
            node.clone()
        } else {
            SyntaxNode::inner(
                node.kind(),
                node.children()
                    .map(|child| self.convert_recursively(child))
                    .collect(),
            )
        }
    }
}

struct TypstDepUpgrader {
    dep: TypstDep,
    ver: Vec<TypstDep>,
}

impl Display for TypstDepUpgrader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> [", self.dep)?;
        for (i, ver) in self.ver.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, " {}", ver.version())?;
        }
        write!(f, " ]")
    }
}

impl TypstDepUpgrader {
    fn build(dep: &TypstDep) -> Self {
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

        Self::build_with_query(dep, |name| TYPST_PACKAGE_META.get(name).cloned())
    }

    fn build_with_query<Q, R>(dep: &TypstDep, query: Q) -> Self
    where
        Q: Fn(&str) -> Option<R>,
        R: IntoIterator<Item = Version>,
    {
        if dep.is_local() {
            panic!("Local package {dep} is not upgradable");
        }

        if dep.namespace() != "preview" {
            panic!("Unknown namespace {} for package {}", dep.namespace(), dep);
        }

        let ver: Vec<_> = (query)(dep.name())
            .expect("Package not found")
            .into_iter()
            .filter(|version| *version > dep.version())
            .map(|version| dep.upgrade_to(version))
            .collect();

        TypstDepUpgrader {
            dep: dep.clone(),
            ver,
        }
    }

    fn next(&self, compatible: bool) -> Option<TypstDep> {
        self.ver
            .iter()
            .filter(|dep| {
                if compatible {
                    dep.version().major == self.dep.version().major
                } else {
                    true
                }
            })
            .max_by_key(|dep| dep.version().clone())
            .cloned()
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::{typstdep::TypstDep, upgrade::TypstDepUpgrader};

    #[test]
    fn next() {
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
        assert_eq!(next_incompat.to_string(), "@preview/package:2.0.0");

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
        assert_eq!(next_compat.to_string(), "@preview/package:1.3.0");

        let next_incompat = upgrader.next(false);
        assert!(next_incompat.is_some());

        let next_incompat = next_incompat.unwrap();
        assert_eq!(next_incompat.to_string(), "@preview/package:2.0.0");
    }

    #[test]
    #[should_panic]
    fn should_not_upgrade_non_preview() {
        let dep = TypstDep::from_str("@non-preview/package:1.2.3").unwrap();
        assert_eq!(dep.namespace(), "non-preview");
        TypstDepUpgrader::build_with_query(&dep, |_| -> Option<Vec<_>> { None });
    }

    #[test]
    #[should_panic]
    fn should_not_upgrade_local() {
        let dep = TypstDep::from_str("@local/package:1.2.3").unwrap();
        assert!(dep.is_local());
        TypstDepUpgrader::build_with_query(&dep, |_| -> Option<Vec<_>> { None });
    }
}
