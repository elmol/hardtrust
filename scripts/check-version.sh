#!/usr/bin/env bash
# check-version.sh — Verify Cargo.toml version matches the release tag.
#
# Usage: RELEASE_VERSION=v0.2.0 bash scripts/check-version.sh
#
# Called by release.yml build jobs. Can also be run locally to debug.

set -euo pipefail

if [ -z "${RELEASE_VERSION:-}" ]; then
  echo "ERROR: RELEASE_VERSION is not set." >&2
  echo "       Usage: RELEASE_VERSION=v0.2.0 bash scripts/check-version.sh" >&2
  exit 1
fi

# Read version from workspace Cargo.toml — works because version.workspace = true
CARGO_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
TAG_VERSION="${RELEASE_VERSION#v}"   # strip leading 'v'

if [ "${CARGO_VERSION}" != "${TAG_VERSION}" ]; then
  echo "ERROR: Cargo.toml version (${CARGO_VERSION}) does not match tag (${TAG_VERSION})" >&2
  echo "       Run 'cargo release' to cut releases — do not push tags manually." >&2
  exit 1
fi

echo "Version check OK: ${CARGO_VERSION}"
