default:
	just --list

@format:
	cargo clippy --fix --allow-dirty --allow-staged -- -D warnings
	cargo fmt --all

@lint:
	cargo fmt --all -- --check
	cargo clippy -- -D warnings

@test:
	echo -e "\e[1m\e[4mCompiling plain package\e[0m\n"
	cargo run -- compile packages/plain

	echo -e "\e[1m\e[4mCompiling counter package\e[0m\n"
	cargo run -- compile -s counter=2 packages/counter

	# FIXME: it's unable to parse the output after 119
	echo -e "\e[1m\e[4mCompiling recursive package\e[0m\n"
	cargo run -- compile --set counter=3 packages/recursive

	echo -e "\n\e[1m\e[4mCompiling with-files package\e[0m\n"
	cargo run -- compile -i packages/with-files/example.json -s 'database.host="not-default"' -s 'database.credentials.user="someone"' packages/with-files

	echo -e "\e[1m\e[4mCompiling with-subpackages package\e[0m\n"
	cargo run -- compile -i packages/with-subpackages/example.json packages/with-subpackages

release:
	#!/usr/bin/env bash
	set -euo pipefail
	VERSION=$(gum input --placeholder "version")
	cargo workspaces version --no-git-commit --exact --yes custom "$VERSION"
	TAG="v${VERSION}"
	git commit -am "Release $TAG"
	git tag -sm "Release $TAG" $TAG
	git push -u origin HEAD
	git push -u origin $TAG

