cargo build --lib --target wasm32-unknown-unknown
wasm-bindgen --debug --target web --out-dir ./target/wasm32/debug target/wasm32-unknown-unknown/debug/wgpudev.wasm
cp target/wasm32/debug/* site/lib/
