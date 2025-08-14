# WebGPU based quantum state vector simulator

**IMPORTANT: WORK IN PROGRESS** and is not yet functional.

The project uses the `wgpu` crate to run a quantum state vector simulation using the GPU on native and web platforms.

## Building the web site

The build requires that wasm-bindgen, wasm-opt, and Node.js are installed and on your PATH.

Run `webdev.sh` to build debug bits for the web. Run `webdev.sh --release` to build release bits.

Run `npx http-server` to serve the site locally. The repo is also served on the project site at <https://ticehurst.com/wgpudev/>.

## Debugging

In debug builds, there is a certain amount of validation and error checking that is done. This can be useful for catching issues early.

Sadly, WGSL does not support any kind of logging yet (see <https://github.com/gfx-rs/wgpu/pull/4297>).

At least on macOS, I've had luck using Xcode as per <https://github.com/gfx-rs/wgpu/wiki/Debugging-with-Xcode>

You could also try the `WebGPU Inspector` Chrome/Edge extension for debugging in the browser. See
<https://github.com/brendan-duncan/webgpu_inspector/blob/main/docs/capture.md> for details on capturing
a workload.

## TODO

- Add the code to configure and dispatch the correct number of workgroups and threads.
- Add the code to run the gate operations.
- Add the code to scan the probabilities and return results over 0.01.
- Update the web page to provide a circuit and show the results.
- Figure out to do logging and capture on large scale simulations for debugging.
