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
    let ising_str = include_str!("ising5x5.crc");
    let circ = Circuit::from_str(ising_str).unwrap();

    // Time start/end duration
    let start = std::time::Instant::now();

    let result = futures::executor::block_on(async {
        let mut gpu_context = gpu_context::GpuContext::new(circ).await;
        gpu_context.create_resources();
        gpu_context.run().await
    });

    let duration = start.elapsed();
    println!("Completed in: {:?}", duration);
}
