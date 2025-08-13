#![allow(unused)]

mod circuit;
mod gpu_context;
mod shader_types;

use circuit::{Circuit, CircuitOp};
use shader_types::ops;

#[cfg(test)]
mod tests;

fn main() {
    let mut gpu_context = gpu_context::GpuContext::new();

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

    gpu_context.create_resources(circ);
    let results = gpu_context.run();
}
