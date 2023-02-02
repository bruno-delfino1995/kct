# Usage

KCT consists of a CLI which acts upon a specific directory structure named [Kubernetes Configuration Package][kcp]. Here we describe the commands in depth, but remember that there's always `--help` there for you. If you're looking for what you can use within your package, check out the [package documentation][kcp]

<a name="compile"></a>

## Compile

Extract the [Kubernetes objects][compile-objects] from the compiled JSON into STDOUT.

It starts by validating your package, as any other command, then goes to validate the merged results of default values and provided values, if everything is valid it'll try to compile your package. Once the package is compiled into the structure defined by your templates, we extract the objects by [walking the paths][kcp-objects] and render that to STDOUT by using the `List` kind from `kubectl`.

The compilation lives at the core because it's how a package becomes a set of resources, so it's our main focus and most primitive resource. The most basic usage of this command is to feed `kubectl` with the resource definitions to manipulate, as we show below by applying and then deleting the compilation results.

``` bash
# install
kct compile kcp -f values.json | kubectl apply -f -

# uninstall
kct compile kcp -f values.json | kubectl delete -f -
```

[compile-objects]: https://kubernetes.io/docs/concepts/overview/working-with-objects/kubernetes-objects/
[kcp-objects]: ./kcp.md#objects
[kcp]: ./kcp.md
