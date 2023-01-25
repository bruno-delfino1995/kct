default:
	just --list

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

