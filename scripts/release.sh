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
OLD_VERSION=$(scripts/get_version.sh)
cargo release "$BUMP_TYPE" --execute --sign --no-verify
NEW_VERSION=$(scripts/get_version.sh)

# cargo-release exits 0 when the confirmation prompt is declined, so make sure
# the version was actually bumped before touching GitHub releases.
if [ "${NEW_VERSION}" = "${OLD_VERSION}" ]; then
	echo -e "${RED}❗️ Version is still ${OLD_VERSION}, cargo-release did not bump it. Aborting.${END}"
	exit 1
fi

RELEASE_TAG="v${NEW_VERSION}"

if ! git ls-remote --exit-code --tags origin "refs/tags/${RELEASE_TAG}" >/dev/null; then
	echo -e "${RED}❗️ Tag ${RELEASE_TAG} was not pushed to origin. Aborting.${END}"
	exit 1
fi

# Draft releases are not unique per tag, so check the full list (drafts included).
if gh api "repos/{owner}/{repo}/releases" --paginate --jq '.[].tag_name' | grep -qxF "${RELEASE_TAG}"; then
	echo -e "${RED}❗️ A GitHub release for ${RELEASE_TAG} already exists. Aborting.${END}"
	exit 1
fi

echo -e "${GRE}Creating GitHub release '${RELEASE_TAG}'...${END}"
# Build the release body from the commit history with git-cliff (same config
# that produced CHANGELOG.md). --latest = just this tag; --strip header drops
# the "# Changelog" preamble.
NOTES_FILE=$(mktemp)
trap 'rm -f "${NOTES_FILE}"' EXIT
git-cliff --latest --strip header >"${NOTES_FILE}"
gh release create "${RELEASE_TAG}" --title "${RELEASE_TAG}" --notes-file "${NOTES_FILE}" --draft

echo -e "${GRE}Trigger release build...${END}"
gh workflow run release.yml -r main

echo -e "${GRE}Done!${END}"
