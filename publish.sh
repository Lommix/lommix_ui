#! /bin/bash
tmp=$(mktemp -d)

echo "$tmp"

cp -r crates/bevy_hui/* "$tmp"/.
cp -r LICENSE-MIT LICENSE-APACHE README.md "$tmp"/.
sed -i 's|"../../../README.md"|"../README.md"|g' "$tmp"/src/lib.rs

cd $tmp && cargo publish
rm -rf "$tmp"
