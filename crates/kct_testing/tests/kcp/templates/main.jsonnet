local _ = import 'kct.libsonnet';

{
	apiVersion: 'v1',
	kind: 'Secret',
	metadata: {
		name: 'api-settings',
	},
	type: 'Opaque',
	data: {
		input: _.input,
		package: _.package,
		release: _.release,
		files: {
			multiple: _.files('**/*.toml', _.input),
			single: _.files('database.toml', _.input),
			plain: _.files('no-params.txt', _.input),
		},
	},
}
