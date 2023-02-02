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
	plain: _.include('github.com/bruno-delfino1995/plain'),
	counter: _.include('github.com/bruno-delfino1995/counter', { counter: _.input.counter - 100 })
})
