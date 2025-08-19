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
    assert_eq!(circ.ops.len(), 476, "Unexpected number of operations in the ising circuit");
    assert_eq!(circ.ops[1].op_id, RX, "First operation should be RX");
}

#[test]
fn run_bell() {
    let circ = Circuit::from_str("h 0\ncx 0 1\n").expect("Failed to parse circuit");

    let results = futures::executor::block_on(async {
        let mut gpu_context = GpuContext::new(circ).await;
        gpu_context.create_resources();
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

#[test]
fn parse_qir_minimal() {
        let qir = r#"
; ModuleID = 'qir'
source_filename = "qir"

declare void @__quantum__qis__sx__body(%Qubit*)
declare void @__quantum__qis__rz__body(double, %Qubit*)
declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
declare void @__quantum__qis__m__body(%Qubit*, %Result*)
declare void @__quantum__rt__initialize(i8*)

define void @main() #0 {
entry:
    call void @__quantum__rt__initialize(i8* null)
    call void @__quantum__qis__sx__body(%Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__rz__body(double 0.5, %Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
    ret void
}

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="2" }
"#;

        let circ = Circuit::from_qir_str(qir).expect("Failed to parse QIR");
        assert_eq!(circ.qubit_count, 2);
        assert_eq!(circ.ops.len(), 5); // Including the final measure all
        assert_eq!(circ.ops[0].op_id, crate::shader_types::ops::SX);
        assert_eq!(circ.ops[1].op_id, crate::shader_types::ops::RZ);
        assert_eq!(circ.ops[1].angle, 0.5);
        assert_eq!(circ.ops[2].op_id, crate::shader_types::ops::CZ);
        assert_eq!(circ.ops[3].op_id, crate::shader_types::ops::MZ);
}

#[test]
fn qir_hidden_shift() {
    let qir = include_str!("hidden_shift.qir");
    let circ = Circuit::from_qir_str(qir).expect("Failed to parse QIR");
    assert_eq!(circ.qubit_count, 6);

    let results = futures::executor::block_on(async {
        let mut gpu_context = GpuContext::new(circ).await;
        gpu_context.create_resources();
        gpu_context.run().await
    });

    assert_eq!(results.len(), 100, "Expected 100 results from the QIR circuit run");
}