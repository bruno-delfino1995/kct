local valid = import 'valid.jsonnet';

function(values, files) {
	imported: valid(values),
}
