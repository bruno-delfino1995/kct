# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- compile command to build your KCP
- help options on CLI with the help of Clap
- provision of values through file or stdin
- support for jsonnet bundler by including `vendor` as search path
- values validation using JSON Schema extracted from `values.schema.json`
- values injection through TLA
- package description from `kcp.json`
