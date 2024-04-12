# Typst Upgrade

[![](https://img.shields.io/crates/v/typst-upgrade
)](https://crates.io/crates/typst-upgrade) [![](https://img.shields.io/github/license/Coekjan/typst-upgrade
)](https://github.com/Coekjan/typst-upgrade)

Help you to upgrade your Typst Packages.

## Usage

To detect the upgradeable packages in your typst project, run the following command (assuming `main.typ` is the entry point of your project):

```console
$ typst-upgrade run main.typ -d
```

> dry-run (`-d` or `--dry-run`) is required now since in-place upgrade is not implemented yet

## Use GitHub Token for More API Rate Limit

As GitHub API has a [rate limit](https://docs.github.com/en/rest/using-the-rest-api/troubleshooting-the-rest-api?apiVersion=2022-11-28#rate-limit-errors), non-authenticated requests are limited to 60 requests per hour. To increase the rate limit, you can [create a GitHub token](https://github.com/settings/tokens) and create `$HOME/.config/typst-upgrade.toml` with the following content:

```toml
token = '<YOUR-GITHUB-TOKEN>'
```

You can also use `typst-upgrade config` to set the token instead of manually editing the config file:

```console
$ typst-upgrade config --token='<YOUR-GITHUB-TOKEN>'
```

You can specify the location of the config file (toml format) in two ways:
- by using `-c` or `--config` option:
  ```console
  $ typst-upgrade -c path/to/file config ...
  ```
- by setting `TYPST_UPGRADE_CONFIG` environment variable:
  ```console
  $ export TYPST_UPGRADE_CONFIG=path/to/file
  $ typst-upgrade config ...
  ``` 

## License

Licensed under the MIT License.
