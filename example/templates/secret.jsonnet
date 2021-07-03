{
	apiVersion: 'v1',
	kind: 'Secret',
	metadata: {
		name: 'api-settings',
	},
	type: 'Opaque',
	data: {
		values: _.values,
		package: _.package,
		release: _.release,
		files: {
			multiple: _.files("**/*.toml"),
			single: _.files("database.toml"),
			plain: _.files("no-params.txt"),
		},
	},
}
