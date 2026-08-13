#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
web_crate="${repo_root}/crates/csv_sculptor_web"
output_dir="${repo_root}/dist"
temp_root="$(mktemp -d "${TMPDIR:-/tmp}/csv_sculptor_web.XXXXXX")"
trap 'rm -rf "$temp_root"' EXIT

wasm-pack build --target web --release --out-dir "${temp_root}/pkg" "${web_crate}" --locked

rm -rf "${output_dir}"
mkdir -p "${output_dir}/pkg"
cp "${web_crate}/static/index.html" "${output_dir}/index.html"
cp "${web_crate}/static/main.js" "${output_dir}/main.js"
cp "${web_crate}/static/styles.css" "${output_dir}/styles.css"
cp "${web_crate}/static/favicon.svg" "${output_dir}/favicon.svg"
cp "${temp_root}/pkg/"* "${output_dir}/pkg/"
touch "${output_dir}/.nojekyll"

test -s "${output_dir}/index.html"
test -s "${output_dir}/pkg/csv_sculptor_web.js"
test -s "${output_dir}/pkg/csv_sculptor_web_bg.wasm"

if find "${output_dir}" -type l -print -quit | grep -q .; then
  printf 'Pages output must not contain symlinks.\n' >&2
  exit 1
fi

printf 'Web output is ready at %s.\n' "${output_dir}"
