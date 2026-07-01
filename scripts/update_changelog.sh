#!/usr/bin/env bash
#
# Regenerates CHANGELOG.md for the version currently being released and stages
# it so cargo-release includes it in the release commit.
#
# Intended to be run by cargo-release as a `pre-release-hook`, which sets
# NEW_VERSION and DRY_RUN in the environment. It can also be run manually
# (defaults to the version in Cargo.toml) to preview the changelog.

set -euo pipefail
cd "$(dirname "$0")/.."

# cargo-release passes the bumped version in NEW_VERSION; fall back to the
# current crate version for manual runs.
VERSION="${NEW_VERSION:-$(scripts/get_version.sh)}"
TAG="v${VERSION}"

if [ "${DRY_RUN:-false}" = "true" ]; then
	echo "🔎 [dry-run] changelog preview for ${TAG}:"
	git-cliff --tag "${TAG}" --unreleased --strip header
	exit 0
fi

echo "📝 Updating CHANGELOG.md for ${TAG}..."
git-cliff --tag "${TAG}" -o CHANGELOG.md
git add CHANGELOG.md
