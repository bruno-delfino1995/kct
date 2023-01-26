default:
	just --list

@format:
	cargo clippy --fix --allow-dirty --allow-staged -- -D warnings
	cargo fmt --all

@lint:
	cargo fmt --all -- --check
	cargo clippy -- -D warnings

@test:
	echo -e "\e[1m\e[4mCompiling simple package\e[0m\n"
	cargo run -- compile packages/simple

	echo -e "\n\e[1m\e[4mCompiling complex package\e[0m\n"
	cargo run -- compile -i packages/with-files/example.json -s 'database.host="not-default"' -s 'database.credentials.user="someone"' packages/with-files

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

