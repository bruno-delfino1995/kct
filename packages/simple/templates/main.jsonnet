{
	apiVersion: "apps/v1",
	kind: "Deployment",
	metadata: {
		name: "debug",
		labels: {
			"app.kubernetes.io/name": "debug"
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
