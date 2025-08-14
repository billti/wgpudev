// @ts-check

import loader from "./lib/wgpudev.js";

async function main() {
    const wasm = await loader();
    console.log("Wasm loaded");

    const runButton = document.getElementById("run");
    runButton?.addEventListener("click", async () => {
        const result = await wasm.run();
        console.log("Results are ", result);
    });

}

window.addEventListener("load", main);
