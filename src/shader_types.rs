#![allow(unused)]

use bytemuck::{Pod, Zeroable};

pub const MAX_QUBITS_PER_THREAD: u32 = 10;
pub const MAX_QUBITS_PER_WORKGROUP: u32 = 12;

// Could use an enum, but this avoids some boilerplate
pub mod ops {
    pub const ID: u32      = 0;
    pub const RESET: u32   = 1;
    pub const X: u32       = 2;
    pub const Y: u32       = 3;
    pub const Z: u32       = 4;
    pub const H: u32       = 5;
    pub const S: u32       = 6;
    pub const S_ADJ: u32   = 7;
    pub const T: u32       = 8;
    pub const T_ADJ: u32   = 9;
    pub const SX: u32      = 10;
    pub const SX_ADJ: u32  = 11;
    pub const RX: u32      = 12;
    pub const RY: u32      = 13;
    pub const RZ: u32      = 14;
    pub const CX: u32      = 15;
    pub const CZ: u32      = 16;
    pub const RZZ: u32     = 17;
    pub const CCX: u32     = 18;
    pub const MZ: u32      = 19;
    pub const MRESETZ: u32 = 20;
    pub const MEVERYZ: u32 = 21; // Implicit at end of circuit (for now)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Op {
    pub op_id: u32,
    pub q1: u32,
    pub q2: u32,
    pub q3: u32, // For ccx
    pub angle: f32, // For rx, ry, rz, rzz
    // Pad out to 256 butes for WebGPU dynamic buffer alignment
    pub padding: [u8; 236],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Result {
    pub entry_idx: u32,
    pub probability: f32,
}
