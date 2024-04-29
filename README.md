# Typst Upgrade

[![](https://img.shields.io/crates/v/typst-upgrade
)](https://crates.io/crates/typst-upgrade) [![](https://img.shields.io/github/license/Coekjan/typst-upgrade
)](https://github.com/Coekjan/typst-upgrade) [![](https://github.com/Coekjan/typst-upgrade/actions/workflows/ci.yml/badge.svg)](https://github.com/Coekjan/typst-upgrade) [![](https://codecov.io/gh/Coekjan/typst-upgrade/graph/badge.svg?token=NV9EOPC4SR)](https://codecov.io/gh/Coekjan/typst-upgrade)

Help you to upgrade your Typst Packages.

## Usage

To upgrade your typst-package dependencies, you can use the following command (assuming your project located in `/path/to/your/project`):

```console
$ typst-upgrade /path/to/your/project
```

See `typst-upgrade --help` for more information:

```console
$ typst-upgrade --help
A tool to upgrade typst packages

Usage: typst-upgrade [OPTIONS] <TYPST_ENTRY_PATHS>...

Arguments:
  <TYPST_ENTRY_PATHS>...  

Options:
  -d, --dry-run        Dry run without editing files
  -i, --incompatible   Allow incompatible upgrades
      --color <COLOR>  [default: auto] [possible values: auto, always, never]
      --diff <DIFF>    [default: short] [possible values: short, full, none]
  -v, --verbose        Print more information
  -h, --help           Print help
  -V, --version        Print version
```

## License

Licensed under the MIT License.
