# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- documentation about motivation, usage, and package structure
- `include` function to render a subpackage under `kcps` with the provided values

### Fixed

- unwanted print on when validating values
- wrong package version on `--version`

## [0.2.0] - 2020-10-23

### Added

- compile KCPs from `.tgz` archives with files at root
- `package` command to create `.tgz` archives for valid KCPs
- `files` function to the global for compiling files with Jinja like engine
- `--only` and `--except` parameters on `compile` to control which objects should be yielded
- `values.json` file on the KCP structure for defaults
- include `lib` path for package aliasing - inspired by [tanka](https://tanka.dev/tutorial/k-lib#aliasing)

### Changed

- stop using TLAs and use `_` global with the previous TLAs as properties
- rename crates to `kct_$crate` to enable publishing on [crates.io][https://crates.io]
- remove `main` field from `kcp.json` in favor of static `templates/main.jsonnet`

## [0.1.0] - 2020-09-29

### Added

- `compile` command to build your KCP into K8s objects for `kubectl apply`
- help options on CLI with the help of Clap
- provision of values through file or stdin
- support for jsonnet bundler by including `vendor` as search path
- values validation using JSON Schema extracted from `values.schema.json`
- values injection through TLA
- package description from `kcp.json`
- "extensibility" guarantees by forbidding non object paths to K8s objects

[Unreleased]: https://github.com/bruno-delfino1995/kct/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/bruno-delfino1995/kct/compare/v0.2.0...v0.1.0
[0.1.0]: https://github.com/bruno-delfino1995/kct/releases/tag/v0.1.0
