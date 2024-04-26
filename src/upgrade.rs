use std::str::FromStr;

use typst_syntax::{ast::*, SyntaxKind, SyntaxNode};

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
            let Ok(dep) = TypstDep::from_str(&s.to_untyped().text()) else {
                return node.clone();
            };
            if dep.is_local() {
                if self.verbose {
                    eprintln!("Local package {dep} is not upgradable");
                }
                return node.clone();
            }
            let Some(next) = dep.upgrade().next(self.compatible) else {
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
                            SyntaxNode::leaf(SyntaxKind::Str, &format!("\"{}\"", next))
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
