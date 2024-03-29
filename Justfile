default:
	just --list

@format:
	cargo clippy --fix --allow-dirty --allow-staged -- -D warnings
	cargo fmt --all

@lint:
	cargo fmt --all -- --check
	cargo clippy -- -D warnings

@clean:
	fd --no-ignore -t f -e profraw -x rm {}
	fd 'coverage|incremental' -x rm -rf {} \; target
	fd '^kct|^libkct' -x rm -rf {} \; target

@test:
	echo -e "\e[1m\e[4mRendering plain package\e[0m\n"
	cargo run -- render samples/plain

	echo -e "\e[1m\e[4mRendering counter package\e[0m\n"
	cargo run -- render -s counter=2 samples/counter

	# FIXME: it's unable to parse the output after 119
	echo -e "\e[1m\e[4mRendering recursive package\e[0m\n"
	cargo run -- render --set counter=3 samples/recursive

	echo -e "\n\e[1m\e[4mRendering with-files package\e[0m\n"
	cargo run -- render -i samples/with-files/example.json -s 'database.host="not-default"' -s 'database.credentials.user="someone"' samples/with-files

	echo -e "\e[1m\e[4mRendering with-subpackages package\e[0m\n"
	cargo run -- render -i samples/with-subpackages/example.json samples/with-subpackages

coverage:
	#!/usr/bin/env bash
	set -euo pipefail
	export CARGO_INCREMENTAL=0
	export RUSTFLAGS="-Cinstrument-coverage"
	export LLVM_PROFILE_FILE="kct-%p-%m.profraw"
	cargo test --tests
	grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/ --ignore 'target/*'
	echo "Coverage report generated at: ./target/debug/coverage/index.html"
	fd --no-ignore -t f -e profraw -x rm {}

release:
	#!/usr/bin/env bash
	set -euo pipefail
	VERSION=$(gum input --placeholder "version")
	TAG="v${VERSION}"
	git switch main
	git commit -am "Release $TAG"
	git tag -sm "Release $TAG" $TAG
	git push origin HEAD
	git push origin $TAG

