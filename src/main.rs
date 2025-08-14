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
    // let ising_str = include_str!("ising5x5.crc");
    // let circ = Circuit::from_str(ising_str).unwrap();

    let circ = Circuit::from_str("
h 0
cx 1 0
").expect("Failed to parse circuit");

    let result = futures::executor::block_on(async {
        let mut gpu_context = gpu_context::GpuContext::new(circ).await;
        gpu_context.create_resources();
        gpu_context.run().await
    });
    println!("Result length: {}", result.len());
}
