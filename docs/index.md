# Welcome

Welcome to the KCT wiki!

Here lives our initial documentation about concepts and how the project works

<a name="about"></a>

## About

KCT is a tool for taming the Kubernetes configuration beast by using [Jsonnet](http://jsonnet.org/) while borrowing approaches and concepts from early contestants such as [Tanka](https://helm.sh/) and [Helm](https://helm.sh/).

<a name="motivation"></a>

## Motivation

The motivation came from ["The State of Kubernetes Configuration Management: An Unsolved Problem"][state-k8s-config] and ["Why the fuck are we templating YAML?"][hideous-templates]. In this project, we avoid [hideous](https://helm.sh/)/[complex](https://kapitan.dev/) templates and [context](https://qbec.io/)/[cluster](https://tanka.dev/) management, those achieved by using Jsonnet as a template engine and leveraging kubeconfig for cluster access.

We want to trust the users with context and cluster access, by using their [preffered tools](https://github.com/ahmetb/kubectx) to manage kubeconfig, while focusing on the creation of the resources with a better templating language.

<a name="roadmap"></a>

## Roadmap

We don't have a clear set of features we want to have, but we do have the path we want to take. You can get a glimpse of such path by looking at our [milestones](https://github.com/kseat/kct/milestones). As summary, we're aspiring to be a Helm alternative but with: optional releases, file templating, Jsonnet, build stages, and more.

<a name="acknowledgments"></a>

## Acknowledgments

Once this project wouldn't be possible without their parser and evaluator, I owe special thanks to the creators of [jrsonnet](https://github.com/CertainLach/jrsonnet).

Thanks to the ArgoCD project for their post, and thanks to the creators of the reviewed tools. I wouldn't be able to know at which features I should aim without their review and prior approaches.

[state-k8s-config]: https://blog.argoproj.io/the-state-of-kubernetes-configuration-management-d8b06c1205
[hideous-templates]: http://leebriggs.co.uk/blog/2019/02/07/why-are-we-templating-yaml.html
