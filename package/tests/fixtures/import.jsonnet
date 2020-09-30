local valid = import 'valid.jsonnet';

function(values) {
	imported: valid(values),
}
