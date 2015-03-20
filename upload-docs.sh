#!/usr/bin/env bash

rev=$(git rev-parse --short HEAD)

set -e

cd target/doc

# Add a page to redirect to the main crate.
echo '<meta http-equiv="refresh" content="0; url=radix_trie/index.html">' > index.html

git init
git config user.name "Michael Sproul"
git config user.email "micsproul@gmail.com"

git remote add upstream "https://$GH_TOKEN@github.com/michaelsproul/rust_radix_trie.git"
git fetch upstream && git reset upstream/gh-pages

git add -A .
git commit -m "Rebuild pages at ${rev}."
git push -q upstream HEAD:gh-pages
