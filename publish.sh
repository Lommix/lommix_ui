#! /bin/bash
tmp=$(mktemp -d)
cp -r crates/bevy_hui/* "$tmp"/.
cp -r LICENSE-MIT LICENSE-APACHE README.md "$tmp"/.
sed -i 's|"../../../README.md"|"../README.md"|g' "$tmp"/src/lib.rs
cd $tmp && cargo publish
rm -rf "$tmp"


tmp=$(mktemp -d)
cp -r crates/bevy_hui_widgets/* "$tmp"/.
cp -r LICENSE-MIT LICENSE-APACHE "$tmp"/.
cd $tmp && cargo publish
rm -rf "$tmp"
