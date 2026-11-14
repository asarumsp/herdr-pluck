#!/usr/bin/env sh
# Build the plugin binary from the checked-out source.
#
# This fork never downloads release binaries: upstream assets are unverified and would not
# contain the fork's fixes.
set -eu

BIN_DIR="bin"
BIN_PATH="$BIN_DIR/herdr-pluck"

if ! command -v cargo >/dev/null 2>&1; then
    printf '%s\n' "herdr-pluck build requires cargo" >&2
    exit 1
fi

cargo build --release
mkdir -p "$BIN_DIR"
cp target/release/herdr-pluck "$BIN_PATH.tmp"
chmod +x "$BIN_PATH.tmp"
mv -f "$BIN_PATH.tmp" "$BIN_PATH"
printf '%s\n' "Installed $BIN_PATH from local source" >&2
