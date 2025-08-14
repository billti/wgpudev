// See https://webgpufundamentals.org/webgpu/lessons/webgpu-wgsl.html for an overview
// See https://www.w3.org/TR/WGSL/ for the details

// ***** IMPORTANT: Keep this first section in sync with the shader_types module in main.rs *****

const MAX_QUBITS_PER_THREAD: u32 = 10u;
const MAX_QUBITS_PER_WORKGROUP: u32 = 12u;

const ID: u32      = 0;
const RESET: u32   = 1;
const X: u32       = 2;
const Y: u32       = 3;
const Z: u32       = 4;
const H: u32       = 5;
const S: u32       = 6;
const S_ADJ: u32   = 7;
const T: u32       = 8;
const T_ADJ: u32   = 9;
const SX: u32      = 10;
const SX_ADJ: u32  = 11;
const RX: u32      = 12;
const RY: u32      = 13;
const RZ: u32      = 14;
const CX: u32      = 15;
const CZ: u32      = 16;
const RZZ: u32     = 17;
const CCX: u32     = 18;
const MZ: u32      = 19;
const MRESETZ: u32 = 20;

struct Op {
    op_id: u32,
    qubit: u32,
    control: u32,
    angle: f32,
}

struct Result {
    entry_idx: u32,
    probability: f32,
}

struct RunInfo {
    shot_buffer_entries: u32,
    qubit_count: u32,
    shot_count: u32,
    output_states_per_thread: u32,
    threads_per_workgroup: u32,
    workgroups: u32,
    op_count: u32,
    op_index: u32,
}

// ***** END IMPORTANT SECTION *****

// Input to the shader. The length of the array is determined by what buffer is bound.
//
// StateVector entries 
@group(0) @binding(0)
var<storage, read_write> stateVec: array<vec2f>;
// Circuit ops.  
@group(0) @binding(1)
var<storage, read> circuitOps: array<Op>;

// Results
@group(0) @binding(2)
var<storage, read_write> results: array<Result>;

override ENTRIES_PER_THREAD: u32 = 10;
override WORKGROUP_SIZE_X: u32 = 32;

// Ideal workgroup size depends on the hardware, the workload, and other factors. However, it should
// _generally_ be a multiple of 64. Common sizes are 64x1x1, 256x1x1; or 8x8x1, 16x16x1 for 2D workloads.
@compute @workgroup_size(WORKGROUP_SIZE_X)
fn run_statevector_ops(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // This will end up being a linear index of all the threads run total (including across workgroups).
    let thread_id = global_id.x + global_id.y * WORKGROUP_SIZE_X;

    let array_length = arrayLength(&results);
    if (thread_id >= array_length) {
        return;
    }

    // Results array will basically be an increasing counter.
    results[thread_id].entry_idx = thread_id;
    results[thread_id].probability = 1.5 * f32(global_id.x);
}
