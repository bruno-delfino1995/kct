# Kubernetes Configuration Tool

> No more context babysitting or hideous templates

A combination from Helm + Tanka for the best of both worlds.

## Motivation

Inspired by the [current state of Kubernetes configuration management](https://blog.argoproj.io/the-state-of-kubernetes-configuration-management-d8b06c1205), I'm giving my take on this matter. I created this in an effort to run away from [hideous](https://helm.sh/)/[complex](https://kapitan.dev/) templates and [context](https://qbec.io/)/[cluster](https://tanka.dev/) babysitting.

By combining the value injection from Helm with the Jsonnet template engine from Tanka, I intend to have a tool that doesn't have static configurations through files but through CLI parameters. Although, differently from Helm alternatives, I still want to trust the developer for context and cluster switching and having the tool only manage packages and their installation on clusters of choice. And taking inspiration from Tanka, I choose Jsonnet since it's a data templating language, instead of the [ugly YAML templating](http://leebriggs.co.uk/blog/2019/02/07/why-are-we-templating-yaml.html), and have mechanisms aiming at object extension/patching which helps with the extension problems of charts from Helm.

## Installation

There are three ways you can install our tool:

- From the binary releases that can be found on the [Releases Page](https://github.com/bruno-delfino1995/kct/releases) and adding the bin to your `$PATH`
- By installing the binary at [crates.io](https://crates.io/crates/kct) with `cargo install kct`
- Through your prefered package manager for your distro:
  - Arch user with `yay -S kct`

## Usage

As we are at the early stages of the project we only provide one command to help you manage your kubernetes objects, although it's up to you how you'll install them.

The recommended usage boils down to compiling with `kct` and installing with `kubectl`:

``` bash
# install
kct compile kcp -f values.json | kubectl apply -f -
# uninstall
kct compile kcp -f values.json | kubectl delete -f -
```

These commands act upon what we called __Kubernetes Configuration Packages (KCP)__ which are collections of templates to build your K8s objects.

### Kubernetes Configuration Packages

This is the "configuration unit" for `kct` much like what __Charts__ are to `helm`.

The requirements for a KCP are:

- A `kcp.json` file describing the package
- A `values.schema.json` if you intend to receive values

We still don't have a mechanism to load Jsonnet libraries and for that we recommend [Jsonnet Bundler](https://github.com/jsonnet-bundler/jsonnet-bundler) at the moment. Jsonnet bundler also helps with subpackages because a KCP is only a collection of jsonnet files, you just have to import the correct entrypoint and you're ready to go.

## Contributing

Any contribution is welcome, be either an issue or a PR. I'm very new to Rust so anything in the code that seems wrong feel free to point it out, your contribution will be very helpful to raise the bar and help my evolution.

## LICENSE

MIT © Bruno Delfino
