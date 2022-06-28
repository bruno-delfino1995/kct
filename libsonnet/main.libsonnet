local input = std.extVar("kct.io/input");
local package = std.extVar("kct.io/package");
local release = std.extVar("kct.io/release");
local files = std.extVar("kct.io/files");
local include = std.extVar("kct.io/include");

{
	name: if release != null then '%s-%s' % [release.name, package.name] else package.name,
	input: input,
	package: package,
	release: release,
	files: files,
	include: include,
	sdk: import 'sdk.libsonnet',
}
