use std::{collections::HashMap, str::FromStr};

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
    upgrader_builder: Box<dyn Fn(&TypstDep) -> TypstDepUpgrader>,
}

impl<'a> TypstNodeUpgrader<'a> {
    #[cfg(not(tarpaulin_include))]
    pub fn new(root: &'a SyntaxNode, verbose: bool, compatible: bool) -> Self {
        Self::new_with_upgrader_builder(root, verbose, compatible, TypstDepUpgrader::build)
    }

    fn new_with_upgrader_builder(
        root: &'a SyntaxNode,
        verbose: bool,
        compatible: bool,
        upgrader_builder: impl Fn(&TypstDep) -> TypstDepUpgrader + 'static,
    ) -> Self {
        Self {
            root,
            verbose,
            compatible,
            upgrader_builder: Box::new(upgrader_builder),
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
                    warn!(
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
                    warn!("NOTE": "Local package {dep} is not upgradable");
                }
                return node.clone();
            }
            let Some(next) = (self.upgrader_builder)(&dep).next(self.compatible) else {
                if self.verbose {
                    warn!("NOTE": "Package {dep} is already up-to-date");
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

impl TypstDepUpgrader {
    #[cfg(not(tarpaulin_include))]
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
    use std::{fs, str::FromStr};

    use paste::paste;
    use semver::Version;

    use crate::{typstdep::TypstDep, upgrade::TypstDepUpgrader};

    use super::TypstNodeUpgrader;

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

    #[test]
    fn upgrader_build() {
        let dep = TypstDep::from_str("@preview/pack1:0.1.0").unwrap();
        let upgrader = TypstDepUpgrader::build_with_query(&dep, mock_query);
        assert_eq!(
            upgrader.next(true).unwrap().to_string(),
            "@preview/pack1:0.2.2"
        );
        assert_eq!(
            upgrader.next(false).unwrap().to_string(),
            "@preview/pack1:1.1.1"
        );
    }

    #[test]
    #[should_panic]
    fn should_not_convert_illegal_root() {
        let root = typst_syntax::parse_math("$1 + 2$");
        TypstNodeUpgrader::new(&root, false, true).convert();
    }

    macro_rules! ex_test {
        ($(#[$attr:meta])* $name:ident / $ext:literal) => {
            paste! {
                #[test]
                $(#[$attr])*
                fn [<ex_upgrade_$name>]() {
                    let entry = fs::read_to_string(&format!(
                        "{}/tests/{}/entry.{}",
                        env!("CARGO_MANIFEST_DIR"),
                        stringify!($name),
                        $ext,
                    )).unwrap();

                    let old_tree = if matches!($ext, "typ" | "typst") {
                        typst_syntax::parse(&entry)
                    } else {
                        typst_syntax::parse_code(&entry)
                    };
                    let new_compat = TypstNodeUpgrader::new_with_upgrader_builder(
                        &old_tree,
                        true,
                        true,
                        mock_upgrader_builder,
                    ).convert();
                    let res_compat = fs::read_to_string(&format!(
                        "{}/tests/{}/entry.compat.{}",
                        env!("CARGO_MANIFEST_DIR"),
                        stringify!($name),
                        $ext,
                    )).unwrap();
                    assert_eq!(new_compat.into_text(), res_compat);

                    let new_incompat = TypstNodeUpgrader::new_with_upgrader_builder(
                        &old_tree,
                        true,
                        false,
                        mock_upgrader_builder,
                    ).convert();
                    let res_incompat = fs::read_to_string(&format!(
                        "{}/tests/{}/entry.incompat.{}",
                        env!("CARGO_MANIFEST_DIR"),
                        stringify!($name),
                        $ext,
                    )).unwrap();
                    assert_eq!(new_incompat.into_text(), res_incompat);
                }
            }
        };
        ($($(#[$attr:meta])* $name:ident / $ext:literal, )*) => {
            $( ex_test!($(#[$attr])* $name / $ext); )*
        };
    }

    ex_test! {
        normal1 / "typ",
        normal2 / "typst",
        normal3 / "typc",
        #[should_panic] exception1 / "typ",
        exception2 / "typ",
    }

    fn mock_upgrader_builder(dep: &TypstDep) -> TypstDepUpgrader {
        TypstDepUpgrader::build_with_query(dep, mock_query)
    }

    fn mock_query(name: &str) -> Option<Vec<Version>> {
        match name {
            "pack1" => Some(vec![
                Version::parse("0.1.0").unwrap(),
                Version::parse("0.1.1").unwrap(),
                Version::parse("0.2.0").unwrap(),
                Version::parse("0.2.1").unwrap(),
                Version::parse("0.2.2").unwrap(),
                Version::parse("1.0.0").unwrap(),
                Version::parse("1.0.1").unwrap(),
                Version::parse("1.1.0").unwrap(),
                Version::parse("1.1.1").unwrap(),
            ]),
            "pack2" => Some(vec![
                Version::parse("0.1.0").unwrap(),
                Version::parse("1.0.0").unwrap(),
                Version::parse("1.1.0").unwrap(),
                Version::parse("2.0.0").unwrap(),
            ]),
            "pack3" => Some(vec![
                Version::parse("0.1.0").unwrap(),
                Version::parse("0.2.0").unwrap(),
                Version::parse("1.0.0").unwrap(),
                Version::parse("2.0.0").unwrap(),
                Version::parse("3.0.0").unwrap(),
            ]),
            _ => None,
        }
    }
}
