local _ = import 'kct.libsonnet';

if _.input.counter == 0 then
	_.include('github.com/bruno-delfino1995/plain')
else
	{
		counter: _.include('github.com/bruno-delfino1995/counter', _.input),
		next: _.include('github.com/bruno-delfino1995/recursive', { counter: _.input.counter - 1 })
	}
