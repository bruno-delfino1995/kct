local _ = import 'kct.io';

{
	apiVersion: 'v1',
	kind: 'Secret',
	metadata: {
		name: _.name,
	},
	type: 'Opaque',
	data: {
		input: _.input,
		package: _.package,
		release: _.release,
		files: {
			multiple: _.files("**/*.toml"),
			single: _.files("database.toml"),
			plain: _.files("no-params.txt"),
		},
	},
}
