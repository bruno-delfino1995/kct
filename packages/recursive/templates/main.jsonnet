local _ = import 'kct.libsonnet';

if _.input.counter == 0 then
	_.include('github.com/oddin-org/plain')
else
	{
		counter: _.include('github.com/oddin-org/counter', _.input),
		next: _.include('github.com/oddin-org/recursive', { counter: _.input.counter - 1 })
	}
