#!/bin/bash

WASM_BUILD_DIR=target/wasm32-unknown-unknown/release
WASM_TARGET="$WASM_BUILD_DIR"/dose-response.wasm
TARGET_WEB_DIR=target/web/

mkdir -p "$TARGET_WEB_DIR"
cp "$WASM_TARGET" "$TARGET_WEB_DIR"/dose-response.wasm
cp web-src/*.js "$TARGET_WEB_DIR"
cp web-src/*.css "$TARGET_WEB_DIR"
cp web-src/*.png "$TARGET_WEB_DIR"
cp web-src/*.glsl "$TARGET_WEB_DIR"
