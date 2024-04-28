use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::{ColorChoice, Parser};

use crate::upgrade::TypstNodeUpgrader;

#[macro_use]
mod term;
mod typstdep;
mod upgrade;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, help = "Dry run without editing files")]
    dry_run: bool,

    #[arg(short, long, help = "Allow incompatible upgrades")]
    incompatible: bool,

    #[arg(long, default_value_t = ColorChoice::Auto)]
    color: ColorChoice,

    #[arg(short, long, help = "Print more information")]
    verbose: bool,

    #[arg(value_name = "TYPST_ENTRY_PATHS", required = true)]
    entries: Vec<PathBuf>,
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        error!("Fatal": "{}", info);
    }));

    let args = Cli::parse();

    term::init(args.color);

    let mut typst_files = args
        .entries
        .iter()
        .flat_map(find_all_typst_files)
        .collect::<Vec<_>>();

    typst_files.sort_unstable();
    typst_files.dedup();

    let typst_files = typst_files;

    for file in &typst_files {
        let ext = file.extension().unwrap();
        let content = fs::read_to_string(file).expect("Cannot read file");
        let tree = if ext == "typ" || ext == "typst" {
            typst_syntax::parse(&content)
        } else if ext == "typc" {
            typst_syntax::parse_code(&content)
        } else {
            panic!("Unknown file extension of: {}", file.display());
        };
        info!("Checking": "{}", file.display());
        let result = TypstNodeUpgrader::new(&tree, args.verbose, !args.incompatible).convert();
        if tree != result {
            let old = tree.into_text();
            let new = result.into_text();
            for diff in diff::lines(&old, &new) {
                match diff {
                    diff::Result::Left(l) => {
                        diff!(del "{}", l);
                    }
                    diff::Result::Right(r) => {
                        diff!(add "{}", r);
                    }
                    _ => (),
                }
            }
            if !args.dry_run {
                info!("Updating": "{}", file.display());
                fs::write(file, new.to_string()).expect("Cannot write file");
            }
        }
    }
}

fn find_all_typst_files(path: impl AsRef<Path>) -> Vec<PathBuf> {
    fn find_all_typst_files_inner(path: impl AsRef<Path>) -> Option<Vec<PathBuf>> {
        let mut result = Vec::new();
        let path = path.as_ref();

        if !path.exists() {
            return None;
        }

        if path.is_dir() {
            for file in fs::read_dir(path).ok()? {
                let Ok(file) = file else {
                    continue;
                };
                if let Some(files) = find_all_typst_files_inner(file.path()) {
                    result.extend(files);
                }
            }
        } else if path.is_symlink() {
            result.extend(find_all_typst_files_inner(fs::read_link(path).ok()?)?);
        } else if path.is_file() {
            if matches!(path.extension()?.to_str()?, "typ" | "typst" | "typc") {
                result.push(path.to_path_buf());
            }
        } else {
            error!("Unknown file type: {}", path.display());
        }

        Some(result)
    }

    if !path.as_ref().exists() {
        panic!("Path does not exist: {}", path.as_ref().display());
    }

    find_all_typst_files_inner(path).unwrap_or_default()
}
