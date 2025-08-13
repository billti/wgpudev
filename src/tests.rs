use crate::circuit::{Circuit};
use crate::shader_types::ops::RX;

#[test]
fn load_ising() {
    // Load the ising5x5.crc file as a string
    let ising_str = include_str!("ising5x5.crc");
    let circ = Circuit::from_str(ising_str).unwrap();

    let qubits = circ.qubit_count;
    assert_eq!(qubits, 25, "Expected 25 qubits in the ising circuit");
    assert_eq!(circ.ops.len(), 475, "Expected 50 operations in the ising circuit");
    assert_eq!(circ.ops[0].op_id, RX, "First operation should be Hadamard");
}