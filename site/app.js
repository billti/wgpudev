// @ts-check

import loader from "./lib/wgpudev.js";
import {run} from "./lib/wgpudev.js";

async function main() {
    await loader();
    console.log("Wasm loaded");

    const runButton = /** @type {HTMLButtonElement} */ (document.getElementById("run"));
    const isingButton = /** @type {HTMLButtonElement} */ (document.getElementById("ising"));
    const circuitTextArea = /** @type {HTMLTextAreaElement} */ (document.getElementById("circuit"));
    const outputCode = /** @type {HTMLElement} */ (document.getElementById("output"));

    runButton.addEventListener("click", async () => {
        // Get the circuit from the textarea
        const circuitText = circuitTextArea.value;

        // Start a performance timer
        const startTime = performance.now();
        const result = await run(circuitText);
        const endTime = performance.now();
        //console.log("Results are ", result)
        //console.log(`Circuit executed in ${(endTime - startTime)} milliseconds`);

        var output = `Runtime: ${(endTime - startTime).toFixed(2)} milliseconds\n\n`;
        for (const [entry_idx, probability] of result) {
            // Convert the bits in entry_idx to a binary string
            if (probability < 0.01) {
                continue; // Skip probabilities less than 0.01%
            }
            // Format probability to 2 decimal places
            const prob_str = (probability * 100).toFixed(4);
            const binaryString = entry_idx.toString(2).padStart(28, '0'); // Assuming 5 qubits
            output += `|${binaryString}>: ${prob_str}%\n`;
        }
        outputCode.innerHTML = output;
    });

    isingButton.addEventListener("click", async () => {
        // Fetch the Ising circuit file from ./src/ising5x5.crc and load into the textarea
        const response = await fetch("./src/ising5x5.crc");
        if (!response.ok) {
            console.error("Failed to fetch the Ising circuit file");
            return;
        }
        const isingCircuit = await response.text();
        circuitTextArea.value = isingCircuit;
    });

}

window.addEventListener("load", main);
