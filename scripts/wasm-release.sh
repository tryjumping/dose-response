#!/bin/bash

# conditionally run wasm gc

WASM_BUILD_DIR=target/wasm32-unknown-unknown/release
WASM_ORIGINAL_TARGET="$WASM_BUILD_DIR"/dose-response.wasm
WASM_GC_TARGET="$WASM_BUILD_DIR"/dose-response-gc.wasm
TARGET_WEB_DIR=target/web/

if command -v wasm-gc >/dev/null 2>&1; then
    echo Found wasm-gc
    wasm-gc -o "$WASM_GC_TARGET" "$WASM_ORIGINAL_TARGET"
else
    echo No wasm-gc found, just copying the file over
    cp "$WASM_ORIGINAL_TARGET" "$WASM_GC_TARGET"
fi

mkdir -p "$TARGET_WEB_DIR"
cp "$WASM_GC_TARGET" "$TARGET_WEB_DIR"/dose-response.wasm
cp *.js *.css "$TARGET_WEB_DIR"
cp "$WASM_BUILD_DIR"/font.png "$TARGET_WEB_DIR"
