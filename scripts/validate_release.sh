#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'Release validation failed: %s\n' "$1" >&2
  exit 1
}

tag="${1:-}"
repo_root="${2:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

if [[ ! "${tag}" =~ ^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)-alpha\.[1-9][0-9]*$ ]]; then
  fail "tag must be an alpha SemVer tag such as v0.1.0-alpha.1"
fi

[[ -d "${repo_root}" ]] || fail "repository root does not exist: ${repo_root}"

for command_name in cargo jq ruby; do
  command -v "${command_name}" >/dev/null 2>&1 || fail "required command is unavailable: ${command_name}"
done

cd "${repo_root}"
version="${tag#v}"
workspace_versions="$(cargo metadata --locked --no-deps --format-version 1 | jq -r '.packages[].version' | sort -u)"
expected_versions="${version}"
if [[ "${workspace_versions}" != "${expected_versions}" ]]; then
  fail "Cargo package versions must all equal ${version}; found: ${workspace_versions//$'\n'/, }"
fi

escaped_version="${version//./\.}"
changelog_count="$(grep -E "^## \[${escaped_version}\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$" CHANGELOG.md | wc -l | tr -d ' ')"
[[ "${changelog_count}" == "1" ]] || fail "CHANGELOG.md must contain exactly one dated section for ${version}"

release_date="$(grep -E "^## \[${escaped_version}\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$" CHANGELOG.md | sed 's/.* - //')"
ruby -rdate -e 'value = ARGV.fetch(0); abort unless Date.iso8601(value).to_s == value' "${release_date}" \
  || fail "CHANGELOG.md release date is not a valid ISO date: ${release_date}"

printf 'Release metadata is consistent for %s (%s).\n' "${tag}" "${release_date}"
