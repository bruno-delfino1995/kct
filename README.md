# Kubernetes Configuration Tool

![build](https://img.shields.io/github/workflow/status/kseat/kct/Code%20Quality)
![license](https://img.shields.io/crates/l/kct)
![version](https://img.shields.io/crates/v/kct?label=version)

KCT is a tool for taming the Kubernetes configuration beast by using Jsonnet while borrowing approaches and concepts from early contestants such as Tanka and Helm.

**NOTICE: This project is under heavy development. Despite the 0.x.y releases being "production ready", don't expect API stability before a 1.0 release as [anything may change](https://semver.org/#spec-item-4) due to experimentation and feedback.**

## Installation

There are three ways you can install our tool:

- By adding a binary releases from the [Releases Page](https://github.com/bruno-delfino1995/kct/releases) to your `$PATH`
- By installing the binary at [crates.io](https://crates.io/crates/kct) with `cargo install kct`
- Through your prefered package manager for your distro:
  - Arch user with `yay -S kct`

## Documentation

If you want to know more about the tool's components and inner workings, take a look at the [documentation](./docs/index.md). There you'll find description on the [package](./docs/kcp.md) structure and feature, along the [commands](./docs/usage.md) with brief explanations about their tasks.

## Contributing

Any contribution is welcome, be either an issue or a PR. I'm very new to Rust so anything in the code that seems wrong feel free to point it out.

## LICENSE

MIT © Bruno Delfino
