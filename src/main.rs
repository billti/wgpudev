#![allow(unused)]

mod circuit;
mod gpu_context;
mod shader_types;
mod wasm;

use circuit::{Circuit};
use shader_types::{ops, Op};

#[cfg(test)]
mod tests;

fn main() {
    let circ = Circuit::from_str("
sx 0
rz(0.5) 1
").expect("Failed to parse circuit");

    let result = futures::executor::block_on(async {
        let mut gpu_context = gpu_context::GpuContext::new().await;
        gpu_context.create_resources(circ);
        gpu_context.run().await
    });
    println!("Result length: {}", result.len());
}
