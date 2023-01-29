local _ = import 'kct.libsonnet';

_.sdk.inOrder(['counter', 'plain'], {
	current: {
		apiVersion: 'v1',
		kind: 'Secret',
		metadata: {
			name: _.name,
		},
		type: 'Opaque',
		data: _.input,
	},
	plain: _.include('github.com/oddin-org/plain'),
	counter: _.include('github.com/oddin-org/counter', { counter: _.input.counter - 100 })
})
