#!/bin/bash

set -e

APP_NAME="miko-rs"
VERSION=$(grep '^version =' Cargo.toml | head -n1 | cut -d '"' -f2)
BIN_DIR="bin"

mkdir -p $BIN_DIR

build_target() {
    local target=$1
    local os=$2
    local arch=$3
    local suffix=$4

    echo "Building for $os/$arch ($target)..."
    
    # Check if cross is installed
    if command -v cross >/dev/null 2>&1; then
        cross build --release --target $target
    else
        rustup target add $target
        cargo build --release --target $target
    fi

    local output_name="${APP_NAME}-${os}-${arch}${suffix}"

    cp "target/${target}/release/${APP_NAME}${suffix}" "${BIN_DIR}/${output_name}"
}

# Linux amd64
build_target "x86_64-unknown-linux-musl" "linux" "amd64" ""

# Linux arm64
build_target "aarch64-unknown-linux-musl" "linux" "arm64" ""

# Mac arm64
if [[ "$OSTYPE" == "darwin"* ]]; then
    build_target "aarch64-apple-darwin" "darwin" "arm64" ""
fi

# Windows amd64
build_target "x86_64-pc-windows-gnu" "windows" "amd64" ".exe"
