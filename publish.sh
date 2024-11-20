#! /bin/bash
tmp=$(mktemp -d)

echo "$tmp"

cp -r crates/bevy_hui/* "$tmp"/.
cp -r LICENSE-MIT LICENSE-APACHE README.md "$tmp"/.

cd $tmp && cargo publish
rm -rf "$tmp"
