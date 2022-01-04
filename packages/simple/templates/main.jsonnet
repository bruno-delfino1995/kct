local _ = import 'kct.io';

{
	apiVersion: "apps/v1",
	kind: "Deployment",
	metadata: {
		name: _.name,
		labels: {
			"app.kubernetes.io/name": "debug",
			package: _.package,
			release: _.release,
		},
	},
	spec: {
		replicas: 1,
		selector: {
			matchLabels: $.metadata.labels,
		},
		template: {
			metadata: {
				labels: $.metadata.labels,
			},
			spec: {
				containers: [
					{
						name: "debug",
						image: "ubuntu:20.04",
						imagePullPolicy: "IfNotPresent",
						command: [ "sh" ],
						args: [ "-c", "sleep infinity" ],
					},
				],
			},
		},
	},
}
