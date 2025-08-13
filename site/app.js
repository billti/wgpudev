// @ts-check

import loader from "./lib/wgpudev.js";

async function main() {
    const wasm = await loader();
    const result = await wasm.run();
    console.log("Result length is ", result);
}

window.addEventListener("load", main);
