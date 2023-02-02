local _ = import 'kct.libsonnet';

_.sdk.inOrder(['namespace', 'deployment'], {
	namespace: {
		apiVersion: "v1",
		kind: "Namespace",
		metadata: {
			name: _.name,
		}
	},
	deployment: {
		apiVersion: "apps/v1",
		kind: "Deployment",
		metadata: {
			name: _.name,
			namespace: $.namespace.metadata.name,
			labels: {
				"app.kubernetes.io/name": "debug",
				package: _.package,
				release: _.release,
			},
		},
		spec: {
			replicas: 1,
			selector: {
				matchLabels: $.deployment.metadata.labels,
			},
			template: {
				metadata: {
					labels: $.deployment.metadata.labels,
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
	},
})
