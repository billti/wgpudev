use crate::circuit::{Circuit, CircuitOp};
use crate::shader_types::ops;
use crate::gpu_context::GpuContext;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub async fn run() -> u32 {
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

    results.len() as u32
}
