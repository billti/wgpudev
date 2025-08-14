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
        // Expect RESET + 3 ops + MEVERYZ (since an m exists, MEVERYZ is skipped)
        // Here we had an m, so no implicit measure; expect 1 (reset) + 3 ops + 1 m = 5 ops total
        assert_eq!(circ.ops.len(), 6); // Including the initial reset and final measure all
        assert_eq!(circ.ops[1].op_id, crate::shader_types::ops::SX);
        assert_eq!(circ.ops[2].op_id, crate::shader_types::ops::RZ);
        assert_eq!(circ.ops[2].angle, 0.5);
        assert_eq!(circ.ops[3].op_id, crate::shader_types::ops::CZ);
        assert_eq!(circ.ops[4].op_id, crate::shader_types::ops::MZ);
}

#[test]
fn qir_hidden_shift() {
    let qir = r#"
%Result = type opaque
%Qubit = type opaque

define void @ENTRYPOINT__main() #0 {
block_0:
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 2 to %Result*))
  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Result* inttoptr (i64 3 to %Result*))
  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Result* inttoptr (i64 4 to %Result*))
  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Result* inttoptr (i64 5 to %Result*))
  call void @__quantum__rt__array_record_output(i64 6, i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 2 to %Result*), i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 3 to %Result*), i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 4 to %Result*), i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 5 to %Result*), i8* null)
  ret void
}

declare void @__quantum__qis__h__body(%Qubit*)

declare void @__quantum__qis__x__body(%Qubit*)

declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)

declare void @__quantum__rt__array_record_output(i64, i8*)

declare void @__quantum__rt__result_record_output(%Result*, i8*)

declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="6" "required_num_results"="6" }
attributes #1 = { "irreversible" }

; module flags

!llvm.module.flags = !{!0, !1, !2, !3}

!0 = !{i32 1, !"qir_major_version", i32 1}
!1 = !{i32 7, !"qir_minor_version", i32 0}
!2 = !{i32 1, !"dynamic_qubit_management", i1 false}
!3 = !{i32 1, !"dynamic_result_management", i1 false}
    "#;

    let circ = Circuit::from_qir_str(qir).expect("Failed to parse QIR");
    assert_eq!(circ.qubit_count, 6);

    let results = futures::executor::block_on(async {
        let mut gpu_context = GpuContext::new(circ).await;
        gpu_context.create_resources();
        gpu_context.run().await
    });

    assert_eq!(results.len(), 100, "Expected 100 results from the QIR circuit run");
}