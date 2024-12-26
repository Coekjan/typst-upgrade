# Typst Upgrade

[![](https://img.shields.io/crates/v/typst-upgrade
)](https://crates.io/crates/typst-upgrade) [![](https://img.shields.io/crates/d/typst-upgrade)](https://crates.io/crates/typst-upgrade) [![](https://img.shields.io/github/license/Coekjan/typst-upgrade
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

### Examples

```console
$ cat main.typ
#import "@preview/cetz:0.2.1"
$ typst-upgrade -i main.typ
    Checking ./main.typ
           - #import "@preview/cetz:0.2.1"
           + #import "@preview/cetz:0.3.1"
    Updating ./main.typ
$ cat main.typ
#import "@preview/cetz:0.3.1"
```

### Compatible Upgrade

By default, `typst-upgrade` will only upgrade your dependencies to the latest compatible version. If you want to upgrade to the latest version regardless of compatibility, you can use the `--incompatible` or `-i` flag.

Typst packages commonly follow [Semantic Versioning](https://semver.org/), so upgrading to the latest compatible version is usually recommended. Note that some packages are in-development (major version is `0`), which means they may introduce breaking changes in minor versions, and `--incompatible` flag is required to upgrade such packages.

### GitHub Actions

You can use `typst-upgrade` in your GitHub Actions workflow to automatically check if any of your dependencies can be upgraded. Here is an example workflow:

```yaml
steps:
  - uses: actions/checkout@v4
  - uses: taiki-e/install-action@cargo-binstall
  - run: cargo binstall typst-upgrade
  - run: typst-upgrade . --dry-run
```

## Installation

### Cargo

You can install `typst-upgrade` via `cargo`:

```console
$ cargo install typst-upgrade
```

Or if you use [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall), you can install `typst-upgrade` via `cargo binstall`:

```console
$ cargo binstall typst-upgrade
```

### Prebuilt Binaries

You can download the prebuilt binaries from the [release page](https://github.com/Coekjan/typst-upgrade/releases).

## License

Licensed under the MIT License.
