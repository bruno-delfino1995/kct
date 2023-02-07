# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `apply` and `delete` commands to interact directly with the current cluster

### Changed

- `compile` is now called `render` because compile itself is inherent to all commands that interact with a package

## [0.6.0] - 2023-02-05

### Added

- order objects at same depth according to their `kind`
- add `set` to `compile` subcommand to help users change specific inputs

### Changed

- no automatic input for `_.files`, now it gets it from the second parameter
- optional input parameter for `_.files` to keep the old experience
- optional input parameter for `_.include` for packages without input
- separate package crate into compiler and jsonnet crates

### Fixed

- `_.include` shared same external vars as parent compiler because of thread_local in jrsonnet

### Removed

- package command was removed because its original purpose is now achieved with jsonnet bundler
- schema validation for `example.json` - you can compile with it as input to validate it

## [0.5.0] - 2022-07-01

### Added

- `_.sdk.inOrder` function to help specify the order in which your objects should be applied
- `kct.io` external variable for the current environment, previously global `_`
- `kct.libsonnet` to export things from `kct.io` more easily along with a future std

### Changed

- refactor compiler to be more pluggable with properties and validators
- refactor bin crate as result of update to clap 3.0
- `fullName` now is an "installation property" under `_.name`
- centralize compilation information

### Removed

- stop compiling archived packages due to lack of dependencies management
- `_` global removed in favor of `kct.io` lib

## [0.4.0] - 2021-12-14

### Added

- allows multiple occurrences of `input`
- enable subpackages to render their own subpackages

### Changed

- subpackages as normal jsonnet dependencies
- change from `_.values` to `_.input` on injected global
- expect `schema.json` instead of `values.schema.json`
- read `default.json` instead of `values.json` for defaults
- no more `default.json`, now there's only `example.json` to enforce explicit inputs

### Fixed

- fix missing conversion from `values` to `input`

## [0.3.1] - 2021-07-03

### Added

- documentation about motivation, usage, and package structure
- `include` function to render a subpackage under `kcps` with the provided values
- `--output` parameter on `compile` to specify where to place compiled artifacts

### Changed

- render yaml multi-documents stream instead of kubectl List object
- `--values|-f` option into `--input|-i` on compile

### Fixed

- unwanted print when validating values
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

[Unreleased]: https://github.com/bruno-delfino1995/kct/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/bruno-delfino1995/kct/compare/v0.6.0...v0.5.0
[0.5.0]: https://github.com/bruno-delfino1995/kct/compare/v0.5.0...v0.4.0
[0.4.0]: https://github.com/bruno-delfino1995/kct/compare/v0.4.0...v0.3.1
[0.3.1]: https://github.com/bruno-delfino1995/kct/compare/v0.3.1...v0.2.0
[0.2.0]: https://github.com/bruno-delfino1995/kct/compare/v0.2.0...v0.1.0
[0.1.0]: https://github.com/bruno-delfino1995/kct/releases/tag/v0.1.0
