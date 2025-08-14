use crate::circuit::{Circuit};
use crate::shader_types::ops;
use crate::gpu_context::GpuContext;
use crate::shader_types::Result;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub async fn run() -> Vec<JsValue> {
    let circ = Circuit::from_str("
h 0
cx 0 1
").expect("Failed to parse circuit");

    let mut gpu_context = GpuContext::new().await;
    gpu_context.create_resources(circ);
    let results = gpu_context.run().await;

    // Convert results to a JS value of an array, with elements being an array (tuple) of entry_idx and probability.
    // We don't have serde, so convert manually.
    let return_val = js_sys::Array::new();
    for result in results {
        let js_tuple = js_sys::Array::new();
        js_tuple.push(&JsValue::from(result.entry_idx));
        js_tuple.push(&JsValue::from(result.probability));
        return_val.push(&js_tuple);
    }
    return_val.to_vec()
}
