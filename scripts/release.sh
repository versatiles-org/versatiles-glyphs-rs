#!/usr/bin/env bash
#
# Usage: ./scripts/bump-release.sh [patch|minor|major]
#
# 1) Checks code.
# 2) Uses cargo-release to bump version and publish to crates.io.
# 3) Pushes commits/tags to GitHub.
# 4) Creates a GitHub release using the new version.

set -euo pipefail
cd "$(dirname "$0")/.."

RED="\033[1;31m"
GRE="\033[1;32m"
END="\033[0m"

if [ -z "${1-}" ]; then
	echo -e "${RED}❗️ Need argument for bumping version: \"patch\", \"minor\" or \"major\"${END}"
	exit 1
fi
BUMP_TYPE="$1"

# 1) Check the code (adjust as needed for your project)
./scripts/check.sh
if [ $? -ne 0 ]; then
	echo -e "${RED}❗️ Check failed!${END}"
	exit 1
fi

# 2) Perform the release.
cargo release "$BUMP_TYPE" --execute --sign --no-verify

RELEASE_TAG=$(cargo get package.version --pretty)
RELEASE_NAME="Release ${RELEASE_TAG}"

echo -e "${GRE}Creating GitHub release '${RELEASE_TAG}'...${END}"
gh release create "${RELEASE_TAG}" --generate-notes --draft

echo -e "${GRE}Trigger release build...${END}"
gh workflow run release.yml -r main

echo -e "${GRE}Done!${END}"
