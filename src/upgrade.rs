use std::{collections::HashMap, str::FromStr};

use once_cell::sync::Lazy;
use typst_syntax::{
    ast::{AstNode, Expr, ModuleImport},
    package::{PackageSpec, PackageVersion},
    SyntaxKind, SyntaxNode,
};

pub struct TypstNodeUpgrader<'a> {
    root: &'a SyntaxNode,
    verbose: bool,
    compatible: bool,
    upgrader_builder: Box<dyn Fn(&PackageSpec) -> PackageUpgrader>,
}

impl<'a> TypstNodeUpgrader<'a> {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn new(root: &'a SyntaxNode, verbose: bool, compatible: bool) -> Self {
        Self::new_with_upgrader_builder(root, verbose, compatible, PackageUpgrader::build)
    }

    fn new_with_upgrader_builder(
        root: &'a SyntaxNode,
        verbose: bool,
        compatible: bool,
        upgrader_builder: impl Fn(&PackageSpec) -> PackageUpgrader + 'static,
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
            SyntaxKind::Markup => self.convert_recursively(self.root),
            kind => panic!("Unexpected node kind: {:?}", kind),
        }
    }

    fn convert_recursively(&self, node: &SyntaxNode) -> SyntaxNode {
        if let Some(module_import) = node.cast::<ModuleImport>() {
            let Expr::Str(s) = module_import.source() else {
                if self.verbose {
                    info!(
                        "NOTE": "Cannot upgrade non-string module import: {}",
                        node.clone().into_text(),
                    );
                }
                return node.clone();
            };
            let Ok(package) = PackageSpec::from_str(&s.get()) else {
                return node.clone();
            };
            if package.namespace == "local" {
                if self.verbose {
                    info!("NOTE": "Local package {package} is not upgradable");
                }
                return node.clone();
            }
            let Some(next) = (self.upgrader_builder)(&package).next(self.compatible) else {
                if self.verbose {
                    info!("NOTE": "Package {package} is already up-to-date");
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

struct PackageUpgrader {
    pkg: PackageSpec,
    ver: Vec<PackageSpec>,
}

impl PackageUpgrader {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn build(package: &PackageSpec) -> Self {
        static TYPST_PACKAGE_META: Lazy<HashMap<String, Vec<PackageVersion>>> = Lazy::new(|| {
            let mut retry_count = 5;
            let resp = loop {
                match reqwest::blocking::get("https://packages.typst.org/preview/index.json") {
                    Ok(resp) => break resp,
                    Err(_) if retry_count > 0 => {
                        retry_count -= 1;
                        warn!(
                            "Failed to fetch package metadata, retrying... ({} attempts left)",
                            retry_count,
                        );
                    }
                    Err(_) => panic!("Failed to fetch package metadata"),
                }
            };
            let raw_meta = resp
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
                    let version = PackageVersion::from_str(
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

        Self::build_with_query(package, |name| TYPST_PACKAGE_META.get(name).cloned())
    }

    fn build_with_query<Q, R>(package: &PackageSpec, query: Q) -> Self
    where
        Q: Fn(&str) -> Option<R>,
        R: IntoIterator<Item = PackageVersion>,
    {
        if package.namespace == "local" {
            panic!("Local package {package} is not upgradable");
        }

        if package.namespace != "preview" {
            panic!(
                "Unknown namespace {} for package {}",
                package.namespace, package
            );
        }

        let ver: Vec<_> = (query)(&package.name)
            .expect("Package not found")
            .into_iter()
            .filter(|version| *version > package.version)
            .map(|version| PackageSpec {
                version,
                ..package.clone()
            })
            .collect();

        PackageUpgrader {
            pkg: package.clone(),
            ver,
        }
    }

    fn next(&self, compatible: bool) -> Option<PackageSpec> {
        self.ver
            .iter()
            .filter(|dep| {
                !compatible
                    || (self.pkg.version.major != 0 && self.pkg.version.major == dep.version.major)
            })
            .max_by_key(|dep| dep.version)
            .cloned()
    }
}

#[cfg(test)]
mod test {
    use std::{fs, str::FromStr};

    use paste::paste;
    use typst_syntax::package::{PackageSpec, PackageVersion};

    use crate::upgrade::PackageUpgrader;

    use super::TypstNodeUpgrader;

    #[test]
    fn next() {
        let package = PackageSpec::from_str("@preview/package:1.2.3").unwrap();

        let upgrader = PackageUpgrader {
            pkg: package.clone(),
            ver: Vec::new(),
        };
        assert!(upgrader.next(true).is_none());
        assert!(upgrader.next(false).is_none());

        let upgrader = PackageUpgrader {
            pkg: package.clone(),
            ver: vec![PackageSpec::from_str("@preview/package:2.0.0").unwrap()],
        };
        assert!(upgrader.next(true).is_none());

        let next_incompat = upgrader.next(false);
        assert!(next_incompat.is_some());

        let next_incompat = next_incompat.unwrap();
        assert_eq!(next_incompat.to_string(), "@preview/package:2.0.0");

        let upgrader = PackageUpgrader {
            pkg: package,
            ver: vec![
                PackageSpec::from_str("@preview/package:1.2.4").unwrap(),
                PackageSpec::from_str("@preview/package:1.3.0").unwrap(),
                PackageSpec::from_str("@preview/package:2.0.0").unwrap(),
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
        let package = PackageSpec::from_str("@non-preview/package:1.2.3").unwrap();
        assert_eq!(package.namespace, "non-preview");
        PackageUpgrader::build_with_query(&package, |_| -> Option<Vec<_>> { None });
    }

    #[test]
    #[should_panic]
    fn should_not_upgrade_local() {
        let package = PackageSpec::from_str("@local/package:1.2.3").unwrap();
        assert!(package.namespace == "local");
        PackageUpgrader::build_with_query(&package, |_| -> Option<Vec<_>> { None });
    }

    #[test]
    fn upgrader_build() {
        let package = PackageSpec::from_str("@preview/pack1:1.1.0").unwrap();
        let upgrader = PackageUpgrader::build_with_query(&package, mock_query);
        assert_eq!(
            upgrader.next(true).unwrap().to_string(),
            "@preview/pack1:1.1.1"
        );
        assert_eq!(
            upgrader.next(false).unwrap().to_string(),
            "@preview/pack1:2.0.0"
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

                    let old_tree = typst_syntax::parse(&entry);
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
                    assert_eq!(new_compat.into_text(), res_compat, concat!("compat: ", stringify!($name), "/", $ext));

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
                    assert_eq!(new_incompat.into_text(), res_incompat, concat!("incompat: ", stringify!($name), "/", $ext));
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
        #[should_panic] exception1 / "typ",
        exception2 / "typ",
    }

    fn mock_upgrader_builder(package: &PackageSpec) -> PackageUpgrader {
        PackageUpgrader::build_with_query(package, mock_query)
    }

    fn mock_query(name: &str) -> Option<Vec<PackageVersion>> {
        match name {
            "pack1" => Some(vec![
                PackageVersion::from_str("0.1.0").unwrap(),
                PackageVersion::from_str("0.1.1").unwrap(),
                PackageVersion::from_str("0.2.0").unwrap(),
                PackageVersion::from_str("0.2.1").unwrap(),
                PackageVersion::from_str("0.2.2").unwrap(),
                PackageVersion::from_str("1.0.0").unwrap(),
                PackageVersion::from_str("1.0.1").unwrap(),
                PackageVersion::from_str("1.1.0").unwrap(),
                PackageVersion::from_str("1.1.1").unwrap(),
                PackageVersion::from_str("2.0.0").unwrap(),
            ]),
            "pack2" => Some(vec![
                PackageVersion::from_str("0.1.0").unwrap(),
                PackageVersion::from_str("1.0.0").unwrap(),
                PackageVersion::from_str("1.1.0").unwrap(),
                PackageVersion::from_str("2.0.0").unwrap(),
            ]),
            "pack3" => Some(vec![
                PackageVersion::from_str("0.1.0").unwrap(),
                PackageVersion::from_str("0.2.0").unwrap(),
                PackageVersion::from_str("1.0.0").unwrap(),
                PackageVersion::from_str("2.0.0").unwrap(),
                PackageVersion::from_str("3.0.0").unwrap(),
            ]),
            _ => None,
        }
    }
}
