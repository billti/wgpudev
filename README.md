# WebGPU based quantum state vector simulator

## TODO

- Run an actual quantum circuit and verify the results for a simple Bell Pair
- Figure out to do logging and capture on large scale simulations
- Create the buffers for the state vector and operations list
- Dispatch workgroups for operations in a loop
- Run a quantum kernel that can do the math
- Be able to fetch and display the results
- See if pollster and flume can be replaced with <https://github.com/rust-lang/futures-rs>

## Next steps

How to represent a list of operations in Rust that can populate the GPU op list.

A vector of some struct seems natural
bytemuck is the crate to reinterpret bytes of things
Need to figure out how to override the workgroup size and the op index for each dispatch.

## Building the web site

The below requires that wasm-bindgen, wasm-opt, and Node.js are installed and on your PATH.

```bash
# Build the initial wasm binary
cargo build --lib --target wasm32-unknown-unknown --release
# Use wasm-bindgen to generate the JavaScript bindings
wasm-bindgen --target web --out-dir ./target/wasm32/release target/wasm32-unknown-unknown/release/wgpudev.wasm
# Optimize the wasm binary
wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --output target/wasm32/release/wgpudev_bg.wasm target/wasm32/release/wgpudev_bg.wasm
# Copy the generated files from target/wasm32/release to the site/lib directory
cp target/wasm32/release/* site/lib/
# To test/view the site (requires Node.js)
npx http-server
```

For debug builds, which will give you useful call stacks in the console, do the below instead:

```bash
cargo build --lib --target wasm32-unknown-unknown
wasm-bindgen --debug --target web --out-dir ./target/wasm32/debug target/wasm32-unknown-unknown/debug/wgpudev.wasm
cp target/wasm32/debug/* site/lib/
```
