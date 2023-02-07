# Usage

KCT consists of a CLI which acts upon a specific directory structure named [Kubernetes Configuration Package][kcp]. Here we describe the commands in depth, but remember that there's always `--help` there for you. If you're looking for what you can use within your package, check out the [package documentation][kcp]

<a name="compile"></a>

## Render

Extract the [Kubernetes objects][k8s-objects] from the compiled JSON into STDOUT.

It starts by validating your package, as any other command, then goes to validate the merged results of provided values, if everything is valid it'll try to compile your package. Once the package is compiled into the structure defined by your templates, we extract the objects by [walking the paths][kcp-objects] and render that to STDOUT as multiple [Yaml Documents](https://www.yaml.info/learn/document.html).

The compilation lives at the core because it's how a package becomes a set of resources, so it's our main focus and most primitive resource. The most basic usage of this command is to feed `kubectl` with the object definitions to manipulate, as we show below by applying and then deleting the compilation results.

```bash
# install
kct render kcp -f values.json | kubectl apply -f -

# uninstall
kct render kcp -f values.json | kubectl delete -f -
```

Another use of the render command is to help you see the diffs between the objects you created by writing them down at a directory of your choice with the `--output|-o` option. For this reason we also officially support a `examples.json` file to showcase you package's input, which could also double as a default input for your diffing needs.

```bash
kct render kcp -f kcp/example.json -o kcp/rendered
```

To make easier to spot changes, we'll use your package layout to determine which paths to put the files in. If your package has a manifest at `grafana.deployment`, that same manifest will be written at `kcp/rendered/granafa/deployment.yml`.

## Apply & Delete

We also have our own apply and delete commands that use `kube-rs` to help us interact with the cluster configured in your `~/.kube/config`. Instead of receiving the target cluster, we rely on the already conventions used by `kubectl`, so all you need is to provide the same inputs as for rendering a package. If the render happens successfully, we'll hapilly apply or delete the objects from your cluster.

```bash
# place your objects in the cluster
kct apply kcp -f values.json

# remove your objects from the cluster
kct apply kcp -f values.json
```

[k8s-objects]: https://kubernetes.io/docs/concepts/overview/working-with-objects/kubernetes-objects/
[kcp-objects]: ./kcp.md#objects
[kcp]: ./kcp.md
