#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{ColorChoice, Parser};
use diffline::DiffChoice;

use crate::upgrade::TypstNodeUpgrader;

#[macro_use]
mod term;
mod diffline;
mod upgrade;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Dry run without editing files, exit with `73` if there are changes
    #[arg(short, long)]
    dry_run: bool,

    /// Allow incompatible upgrades
    #[arg(short, long)]
    incompatible: bool,

    /// Colorize output
    #[arg(long, default_value_t = ColorChoice::Auto)]
    color: ColorChoice,

    /// Diff style
    #[arg(long, default_value_t = DiffChoice::Short)]
    diff: DiffChoice,

    /// Print more information
    #[arg(short, long)]
    verbose: bool,

    /// Typst entry paths
    #[arg(value_name = "TYPST_ENTRY_PATHS", required = true)]
    entries: Vec<PathBuf>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn main() -> ExitCode {
    std::panic::set_hook(Box::new(|info| {
        if let Some(info) = info.payload().downcast_ref::<&str>() {
            error!("Fatal": "{}", info);
        }
    }));

    let args = Cli::parse();

    term::init(args.color);
    diffline::init(args.diff);

    let mut typst_files = args
        .entries
        .iter()
        .flat_map(find_all_typst_files)
        .collect::<Vec<_>>();

    typst_files.sort_unstable();
    typst_files.dedup();

    let typst_files = typst_files;

    let mut exit_code = ExitCode::SUCCESS;
    let mut incompat_versions_available = false;

    for file in &typst_files {
        let ext = file.extension().unwrap();
        let content = fs::read_to_string(file).expect("Cannot read file");
        let tree = if ext == "typ" || ext == "typst" {
            typst_syntax::parse(&content)
        } else {
            panic!("Unknown file extension of: {}", file.display());
        };
        info!("Checking": "{}", file.display());
        let (result, has_incompat_versions) =
            TypstNodeUpgrader::new(&tree, args.verbose, !args.incompatible).convert();
        incompat_versions_available |= has_incompat_versions;
        if tree != result {
            let old = tree.into_text();
            let new = result.into_text();
            diffline::show(&old, &new);
            if args.dry_run {
                exit_code = ExitCode::from(73);
            } else {
                info!("Updating": "{}", file.display());
                fs::write(file, new.to_string()).expect("Cannot write file");
            }
        }
    }

    if incompat_versions_available {
        warn!("Some packages have incompatible versions, apply the update with `--incompatible` or `-i` flag");
    }

    exit_code
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
            if matches!(path.extension()?.to_str()?, "typ" | "typst") {
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

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::find_all_typst_files;

    #[test]
    #[should_panic]
    fn should_not_find_typst_files_in_non_existent_dir() {
        find_all_typst_files("non-existent-dir");
    }

    #[test]
    fn should_find_typst_files_in_proj() {
        let path = Path::new(&env!("CARGO_MANIFEST_DIR").to_string())
            .join("tests")
            .join("proj");
        let files = find_all_typst_files(path);
        assert!(files.iter().all(|f| f.exists() && f.is_file()));
        assert!(files.iter().all(|f| f
            .extension()
            .is_some_and(|ext| matches!(ext.to_str().unwrap(), "typ" | "typst"))));

        // See `${PROJECT_ROOT}/tests/proj/` for the directory structure
        for dir in [
            "lib1",
            "lib2",
            "lib3",
            "ｌｉｂ4", // for utf-8 path testing
            "sublib1",
        ] {
            assert!(files.iter().any(|f| f
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                == dir));
        }
    }
}
