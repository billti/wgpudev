use crate::circuit::Circuit;
use crate::gpu_context::GpuContext;
use crate::shader_types::{ops::RX, Result};

fn f32_close(a: f32, b: f32) -> bool {
    let epsilon =1e-6; // Ensure a reasonable minimum epsilon
    (a - b).abs() < epsilon
}

#[test]
fn load_ising() {
    // Load the ising5x5.crc file as a string
    let ising_str = include_str!("ising5x5.crc");
    let circ = Circuit::from_str(ising_str).unwrap();

    let qubits = circ.qubit_count;
    assert_eq!(qubits, 25, "Expected 25 qubits in the ising circuit");
    assert_eq!(circ.ops.len(), 477, "Unexpected number of operations in the ising circuit");
    assert_eq!(circ.ops[1].op_id, RX, "First operation should be RX");
}

#[test]
fn run_bell() {
    let circ = Circuit::from_str("h 0\ncx 0 1\n").expect("Failed to parse circuit");

    let results = futures::executor::block_on(async {
        let mut gpu_context = GpuContext::new().await;
        gpu_context.create_resources(circ);
        gpu_context.run().await
    });

    assert!(results.len() == 100, "Expected 100 results from the Bell circuit run");

    let first = &results[0];
    let second = &results[1];

    assert!(first.entry_idx == 0 /* |00> */, "First result entry index should be 0");
    assert!(f32_close(first.probability, 0.5), "First result probability should be 50%");
    assert!(second.entry_idx == 3 /* |11> */, "Second result entry index should be 1");
    assert!(f32_close(second.probability, 0.5), "Second result probability should be 50%");
}
