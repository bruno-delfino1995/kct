# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[Unreleased]: https://github.com/bruno-delfino1995/kct/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/bruno-delfino1995/kct/releases/tag/v0.1.0
