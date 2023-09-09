#!/bin/bash
cargo build --release --target=wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/web.wasm --out-dir . --target web --no-typescript
