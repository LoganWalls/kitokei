#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)
out="$script_dir/external/zed"
rm -rf "$out"
mkdir -p "$out"
echo 'Files in this directory are downloaded from the Zed project and carry the associated licenses.' > "$out/README.md"

tmpdir=$(mktemp -d 2> /dev/null || mktemp -d -t 'kitokei')
cd "$tmpdir" || exit 1

GIT_LFS_SKIP_SMUDGE=1 git clone https://github.com/zed-industries/zed/ --depth 1
cd "zed/crates/languages" || exit 1
cp -L LICENSE* "$script_dir/external/zed"

cd "src" || exit 1
# Delete everything except highlight queries
rm -f ./*.rs
shopt -s extglob
rm -f ./**/!(highlights.scm)
shopt -u extglob
cd "../" || exit 1
# Replace the old queries
mv "src/" "$out/queries"

rm -rf "$tmpdir"
