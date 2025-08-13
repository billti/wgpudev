if [ "$1" == "--release" ]; then
    cargo build --lib --target wasm32-unknown-unknown --release
    wasm-bindgen --target web --out-dir ./target/wasm32/release target/wasm32-unknown-unknown/release/wgpudev.wasm
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --output target/wasm32/release/wgpudev_bg.wasm target/wasm32/release/wgpudev_bg.wasm
    cp target/wasm32/release/* site/lib/
else
    cargo build --lib --target wasm32-unknown-unknown
    wasm-bindgen --debug --target web --out-dir ./target/wasm32/debug target/wasm32-unknown-unknown/debug/wgpudev.wasm
    cp target/wasm32/debug/* site/lib/
fi
