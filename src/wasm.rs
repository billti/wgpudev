use crate::circuit::{Circuit, CircuitOp};
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
    let circ = Circuit {
        qubit_count: 4,
        ops: vec![
            CircuitOp {
                op_id: ops::SX,
                q1: 0,
                q2: 0,
                q3: 0,
                arg: 0.0,
                padding: [0; 236],
            },
            CircuitOp {
                op_id: ops::RZ,
                q1: 1,
                q2: 2,
                q3: 3,
                arg: 0.5,
                padding: [0; 236],
            },
        ],
    };

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
