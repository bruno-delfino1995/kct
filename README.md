# Kubernetes Configuration Tool

![build](https://img.shields.io/github/actions/workflow/status/bruno-delfino1995/kct/lints.yml?branch=main)
![license](https://img.shields.io/crates/l/kct)
![version](https://img.shields.io/crates/v/kct?label=version)

KCT is a tool for taming the Kubernetes configuration beast by using Jsonnet while borrowing approaches and concepts from early contestants such as Tanka and Helm.

**NOTICE: This project is under heavy development. Despite the 0.x.y releases being "production ready", don't expect API stability before a 1.0 release as [anything may change](https://semver.org/#spec-item-4) due to experimentation and feedback.**

## Installation

### Releases

We build binaries for most platforms, you can take a look at our [Releases Page](https://github.com/bruno-delfino1995/kct/releases). From there, grab which binary matches your platform and add it to your `$PATH`

### Build from sources

Our minimum supported rust version (MSRV) is the latest stable, and it'll probably stay that way until we think about external extensions. To build it from source, you just need to run:

``` sh
cargo build --bin=kct --release
```

And if you have the cargo bin folder on your path, you can install it directly with:

``` sh
cargo install --path=bin
```

## Documentation

If you want to know more about the tool's components and inner workings, take a look at the [documentation](./docs/index.md). There you'll find description on the [package](./docs/kcp.md) structure and feature, along the [commands](./docs/usage.md) with brief explanations about their tasks.

## Contributing

Any contribution is welcome, be either an issue or a PR. I'm very new to Rust so anything in the code that seems wrong feel free to point it out.

## LICENSE

MIT © Bruno Delfino
