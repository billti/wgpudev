// See https://webgpufundamentals.org/webgpu/lessons/webgpu-wgsl.html for an overview
// See https://www.w3.org/TR/WGSL/ for the details

// NOTE: WGSL doesn't have the ternary operator, but does have a built-in function `select` that can be used to achieve similar functionality.
// See https://www.w3.org/TR/WGSL/#select-builtin

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
const MEVERYZ: u32 = 21;

struct Op {
    op_idx: u32,
    op_id: u32,
    q1: u32,
    q2: u32,
    q3: u32,
    angle: f32,
}

struct Result {
    entry_idx: u32,
    probability: f32,
}

// ***** END IMPORTANT SECTION *****

const M_PI       = 3.14159265358979323846264338327950288;  /* pi        */
const M_PI_2     = 1.57079632679489661923132169163975144;  /* pi/2      */
const M_PI_4     = 0.78539816339744830961566084581987572;  /* pi/4      */
const M_SQRT2    = 1.41421356237309504880168872420969808;  /* sqrt(2)   */
const M_SQRT1_2  = 0.70710678118654752440084436210484903;  /* 1/sqrt(2) */

// Input to the shader. The length of the array is determined by what buffer is bound.
//
// StateVector entries 
@group(0) @binding(0)
var<storage, read_write> stateVec: array<vec2f>;
// Circuit ops.  
@group(0) @binding(1)
var<storage, read> op: Op;

// Results
@group(0) @binding(2)
var<storage, read_write> results: array<Result>;

@group(0) @binding(3)
var<storage, read_write> result_idx: atomic<u32>;

// The below should all be overridden by the Rust code when creating the pipeline based on the circuit
override WORKGROUP_SIZE_X: u32;
override QUBIT_COUNT: u32;

@compute @workgroup_size(WORKGROUP_SIZE_X)
fn run_statevector_ops(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // This will end up being a linear id of all the threads run total (including across workgroups).
    let thread_id = global_id.x + global_id.y * WORKGROUP_SIZE_X;

    // For the last op, the first thread should scan the probabilities and write the results.
    if (op.op_id == MEVERYZ) {
        scan_probabilities(thread_id);
        return;
    }
    // TODO: MZ and MRESETZ (assume base profile with all measurements at the end of the circuit for now)

    switch op.op_id {
        case ID {
            // No operation, just return.
            return;
        }
        case X, Y, Z, H, S, S_ADJ, T, T_ADJ, SX, SX_ADJ, RX, RY, RZ {
            apply_1q_op(thread_id);
            return;
        }
        case CX, CZ, RZZ {
            apply_2q_op(thread_id);
            return;
        }
        default {
            // TODO: Report error for unsupported op
        }
    }
}

fn cplxmul(a: vec2f, b: vec2f) -> vec2f {
    return vec2f(
        a.x * b.x - a.y * b.y,
        a.x * b.y + a.y * b.x
    );
}

fn apply_1q_op(thread_id: u32) {
    const ITERATIONS: i32 = 1 << (MAX_QUBITS_PER_THREAD - 1);

    let stride: i32 = 1 << op.q1;
    let thread_start_iteration: i32 = i32(thread_id) * ITERATIONS;

    // Find the start offset based on the thread and stride
    var offset: i32 = thread_start_iteration % stride + ((thread_start_iteration / stride) * 2 * stride);
    let iterations: i32 = select(ITERATIONS, (1 << (QUBIT_COUNT - 1)), QUBIT_COUNT < MAX_QUBITS_PER_THREAD);

    var coeff1: vec2f = vec2f(0.0, 0.0);
    var coeff2: vec2f = vec2f(0.0, 0.0);

    // TODO: X, Y, Z, S, S_ADJ, T, T_ADJ, SX_ADJ, RY
    switch op.op_id {
        case SX {
            coeff1 = vec2f(0.5, 0.5);
            coeff2 = vec2f(0.5, -0.5);
        }
        case RX {
            coeff1 = vec2f(cos(op.angle / 2.0), 0.0);
            coeff2 = vec2f(0, -sin(op.angle / 2));
        }
        case RZ {
            // Coeff1 is just 1, and don't get used for Rz
            coeff2 = vec2f(cos(op.angle), sin(op.angle));
        }
        case H {
            coeff1 = vec2f(M_SQRT1_2, 0.0);
            coeff2 = vec2f(-M_SQRT1_2, 0.0);
        }
        default {
            // TODO: Error
        }
    }

    for (var i: i32 = 0; i < iterations; i++) {
        let entry1 = stateVec[offset + stride];

        switch op.op_id {
            case X {
                stateVec[offset + stride] = stateVec[offset];
                stateVec[offset] = entry1;
            }
            case SX {
                let entry0 = stateVec[offset];
                let res0 = cplxmul(entry0, coeff1) + cplxmul(entry1, coeff2);
                let res1 = cplxmul(entry0, coeff2) + cplxmul(entry1, coeff1);

                stateVec[offset] = res0;
                stateVec[offset + stride] = res1;
            }
            case RZ {
                let res1 = cplxmul(entry1, coeff2);
                stateVec[offset + stride] = res1;
            }
            case RX {
                let entry0 = stateVec[offset];
                let res0 = cplxmul(entry0, coeff1) + cplxmul(entry1, coeff2);
                let res1 = cplxmul(entry0, coeff2) + cplxmul(entry1, coeff1);

                stateVec[offset] = res0;
                stateVec[offset + stride] = res1;
            }
            case H {
                let entry0 = stateVec[offset];
                let res0 = cplxmul(entry0, coeff1) + cplxmul(entry1, coeff1);
                let res1 = cplxmul(entry0, coeff1) + cplxmul(entry1, coeff2);

                stateVec[offset] = res0;
                stateVec[offset + stride] = res1;
            }
            default {
                // Should never happen, as should have errored in prior switch.
            }
        }

        offset += 1;
        // If we walked past the end of the block, jump to the next stride
        // The target qubit flips to 1 when we walk past the 0 entries, and
        // a target qubit value is also the stride size
        offset += (offset & stride);
    }
}

fn apply_2q_op(thread_id: u32) {
    const ITERATIONS: i32 = 1 << (MAX_QUBITS_PER_THREAD - 2); 

    let iterations: i32 = select(1 << (QUBIT_COUNT - 2), ITERATIONS, QUBIT_COUNT >= MAX_QUBITS_PER_THREAD);
    let start_count: i32 = i32(thread_id) * ITERATIONS;
    let end_count: i32 = start_count + iterations;

    // Coefficient only needed for RZZ
    let coeff: vec2f = select(vec2f(0.0), vec2f(cos(op.angle), -sin(op.angle)), op.op_id == RZZ);

    let lowQubit = select(op.q1, op.q2, op.q1 > op.q2);
    let hiQubit = select(op.q1, op.q2, op.q1 < op.q2);

    let lowBitCount = lowQubit;
    let midBitCount = hiQubit - lowQubit - 1;
    let hiBitCount = QUBIT_COUNT - hiQubit - 1;

    let lowMask = (1 << lowBitCount) - 1;
    let midMask = (1 << (lowBitCount + midBitCount)) - 1 - lowMask;
    let hiMask = (1 << (lowBitCount + midBitCount + hiBitCount)) - 1 - midMask - lowMask;

    for (var i: i32 = start_count; i < end_count; i++) {
        switch op.op_id {
            case CX {
                // q1 is the control, q2 is the target
                let offset10: i32 = (i & lowMask) | ((i & midMask) << 1) | ((i & hiMask) << 2) | (1 << op.q1);
                let offset11: i32 = (i & lowMask) | ((i & midMask) << 1) | ((i & hiMask) << 2) | (1 << op.q1) | (1 << op.q2);

                let old10 = stateVec[offset10];
                stateVec[offset10] = stateVec[offset11];
                stateVec[offset11] = old10;
            }
            case CZ {
                let offset: i32 = (i & lowMask) | (1 << lowQubit) | ((i & midMask) << 1) | (1 << hiQubit) | ((i & hiMask) << 2);
                stateVec[offset] *= -1;
            }
            case RZZ {
                let offset01: i32 = (i & lowMask) | ((i & midMask) << 1) | (1 << hiQubit) | ((i & hiMask) << 2);
                let offset10: i32 = (i & lowMask) | (1 << lowQubit) | ((i & midMask) << 1) | ((i & hiMask) << 2);

                stateVec[offset01] = cplxmul(stateVec[offset01], coeff);
                stateVec[offset10] = cplxmul(stateVec[offset10], coeff);
            }
            default {

            }
        }
    }
}

fn scan_probabilities(thread_id: u32) {
    // Scan the chunk of the state vector assigned to this thread and for any probabilities above 1%,
    // write the result to the results buffer and update the atomic index.
    const ITERATIONS: u32 = 1u << (MAX_QUBITS_PER_THREAD); 

    let iterations: u32 = select(1u << (QUBIT_COUNT), ITERATIONS, QUBIT_COUNT >= MAX_QUBITS_PER_THREAD);
    let start_idx: u32 = thread_id * ITERATIONS;
    let end_idx: u32 = start_idx + iterations;

    for (var i: u32 = start_idx; i < end_idx; i++) {
        // Calculate the probability of this entry
        let entry = stateVec[i];
        let prob = entry.x * entry.x + entry.y * entry.y;
        if prob > 0.01 {
            // Use atomic operations to safely write to the results buffer
            let curr_idx = atomicAdd(&result_idx, 1);
            if curr_idx >= arrayLength(&results) {
                // Shouldn't happen, but just for safety
                continue;
            }
            results[curr_idx].entry_idx = i;
            results[curr_idx].probability = prob;
        }
    }
}