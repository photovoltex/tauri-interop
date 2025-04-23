#!/bin/bash

# change to true to test the script without commiting and publishing
dry=false

upgrade_version() {
  echo "upgrade '$1' to version:"
  read version

  sed -i -E "0,/[0-9]+\.[0-9]+\.[0-9]+/ s/[0-9]+\.[0-9]+\.[0-9]+/$version/" $2

  if [ "$2" != "Cargo.toml" ]; then
    echo "upgrading version usage in root 'Cargo.toml'"
    sed -i -E "s/tauri-interop-macro = \{ version = \"[0-9]+\.[0-9]+\.[0-9]+\"/tauri-interop-macro = \{ version = \"$version\"/" Cargo.toml
    git add Cargo.toml
  fi

  if [ $dry = true ]; then
    cargo publish --all-features --dry-run --allow-dirty --package $1
  else
    git add $2
    git commit -m "v$version: $1"
    cargo publish --all-features --package $1
  fi
}

echo "upgrade macro version? (Y/n)"
read upgrade_macro

if [ "$upgrade_macro" == "Y" ] || [ "$upgrade_macro" == "y" ] || [ -z $upgrade_macro  ]; then
  upgrade_version "tauri-interop-macro" "tauri-interop-macro/Cargo.toml"
fi

upgrade_version "tauri-interop" "Cargo.toml"
if [ $dry = false ]; then
  git tag "v$version"
fi
