# Typst Upgrade

[![](https://img.shields.io/crates/v/typst-upgrade
)](https://crates.io/crates/typst-upgrade) [![](https://img.shields.io/github/license/Coekjan/typst-upgrade
)](https://github.com/Coekjan/typst-upgrade)

Help you to upgrade your Typst Packages.

## Usage

To detect the upgradeable packages in your typst project, run the following command (assuming `main.typ` is the entry point of your project):

```console
$ typst-upgrade main.typ -d
```

> dry-run (`-d` or `--dry-run`) is required now since in-place upgrade is not implemented yet

## License

Licensed under the MIT License.
