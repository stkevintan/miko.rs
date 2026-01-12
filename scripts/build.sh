#!/bin/bash

set -e

APP_NAME="miko-rs"
VERSION=$(grep '^version =' Cargo.toml | head -n1 | cut -d '"' -f2)
BIN_DIR="bin"

SKIP_FRONTEND=false
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --skip-frontend) SKIP_FRONTEND=true ;;
        *) break ;;
    esac
    shift
done

if [ "$SKIP_FRONTEND" = false ]; then
    echo "Building frontend..."
    pnpm build
fi

mkdir -p "$BIN_DIR"

build_target() {
    local target=$1
    local os=$2
    local arch=$3
    local suffix=$4

    echo "Building for $os/$arch ($target)..."
    
    # Check if cross is installed
    if command -v cross >/dev/null 2>&1; then
        cross build --release --target "$target"
    else
        rustup target add "$target" || true
        cargo build --release --target "$target"
    fi

    local output_name="${APP_NAME}-${os}-${arch}-${VERSION}${suffix}"
    cp "target/${target}/release/${APP_NAME}${suffix}" "${BIN_DIR}/${output_name}"
}

if [ $# -ge 1 ] && [[ "$1" == *"/"* ]]; then
    IFS_BACKUP=$IFS
    IFS='/' read -r OS ARCH <<< "$1"
    IFS=$IFS_BACKUP
    
    SUFFIX=""
    case $OS in
        linux)
            if [ "$ARCH" = "amd64" ]; then
                TARGET="x86_64-unknown-linux-musl"
            elif [ "$ARCH" = "arm64" ]; then
                TARGET="aarch64-unknown-linux-musl"
            fi
            ;;
        darwin)
            if [ "$ARCH" = "arm64" ]; then
                TARGET="aarch64-apple-darwin"
            fi
            ;;
        windows)
            if [ "$ARCH" = "amd64" ]; then
                TARGET="x86_64-pc-windows-gnu"
                SUFFIX=".exe"
            fi
            ;;
    esac

    if [ -n "$TARGET" ]; then
        build_target "$TARGET" "$OS" "$ARCH" "$SUFFIX"
        exit 0
    else
        echo "Unknown platform: $1"
        exit 1
    fi
fi
elif [ $# -eq 4 ]; then
    TARGET=$1
    OS=$2
    ARCH=$3
    SUFFIX=$4

    build_target "$TARGET" "$OS" "$ARCH" "$SUFFIX"
else 
    echo "Building for all targets..."

    build_target "x86_64-unknown-linux-musl" "linux" "amd64" ""
    build_target "aarch64-unknown-linux-musl" "linux" "arm64" ""
if [[ "$OSTYPE" == "darwin"* ]]; then
    build_target "aarch64-apple-darwin" "darwin" "arm64" ""
fi
    build_target "x86_64-pc-windows-gnu" "windows" "amd64" ".exe"
fi
