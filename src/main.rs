use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;
use typst_syntax::{
    ast::{Expr, ModuleImport},
    SyntaxKind, SyntaxNode,
};

use crate::typstdep::{TypstDep, TypstDepUpgrader};

mod typstdep;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    dry_run: bool,

    #[arg(short, long)]
    incompatible: bool,

    #[arg(value_name = "TYPST_ENTRY_FILE")]
    entry: PathBuf,

    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Cli::parse();

    fn find_all_module_import(node: &SyntaxNode) -> Vec<SyntaxNode> {
        let mut mods = Vec::new();

        fn find_module_import(node: &SyntaxNode, result: &mut Vec<SyntaxNode>) {
            if matches!(node.kind(), SyntaxKind::ModuleImport) {
                result.push(node.clone());
            }

            for child in node.children() {
                find_module_import(child, result);
            }
        }

        find_module_import(node, &mut mods);
        mods
    }

    let entry = fs::canonicalize(args.entry).unwrap();
    if args.verbose {
        eprintln!("Start to find deps from: {:?}", entry);
    }

    let mut deps: HashMap<PathBuf, Vec<TypstDep>> = HashMap::new();
    let mut files = VecDeque::new();
    files.push_back(entry);

    while let Some(file) = files.pop_front() {
        let tree = typst_syntax::parse(&fs::read_to_string(&file).unwrap());

        let module_imports = find_all_module_import(&tree);

        for mod_import in module_imports
            .iter()
            .map(|node| node.cast::<ModuleImport>().unwrap())
        {
            match mod_import.source() {
                Expr::Str(s) => {
                    let source = s.get();
                    if let Ok(dep) = TypstDep::from_str(&source) {
                        if args.verbose {
                            eprintln!("Found dependency: {}", dep);
                        }
                        deps.entry(file.clone()).or_default().push(dep);
                    } else {
                        let path = file
                            .parent()
                            .unwrap()
                            .join(PathBuf::from_str(&source).unwrap());
                        if Path::new(&path).exists() {
                            if args.verbose {
                                eprintln!("Found module {:?} at: {:?}", source, path);
                            }
                            files.push_back(fs::canonicalize(path).unwrap());
                        } else {
                            eprintln!("Cannot find module {:?} at: {:?}", source, path);
                        }
                    }
                }
                other => panic!("non-string module import: {:?}", other),
            }
        }
    }

    if args.verbose {
        eprintln!("Start to detect upgradable dependencies");
    }

    let mut upgraders: HashMap<TypstDep, TypstDepUpgrader> = HashMap::new();
    for dep in deps.values().flatten() {
        if !upgraders.contains_key(dep) {
            upgraders.insert(dep.clone(), dep.upgrade());
        }
    }

    for (file, dep) in deps.iter() {
        println!("{}:", file.display());
        for d in dep {
            print!("  {}", d);
            if let Some(upgrader) = upgraders.get(d) {
                match upgrader.next(!args.incompatible) {
                    Some(next) => println!(" -> {}", next),
                    None => println!(" -> (already latest)"),
                }
            }
        }
    }

    if !args.dry_run {
        todo!("Edit files with new dependencies")
    }
}
