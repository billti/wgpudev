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
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Op {
    pub op_id: u32,
    pub target: u32,
    pub control: u32,
    pub control2: u32, // For ccx
    pub angle: f32, // For rx, ry, rz, rzz
    // Pad out to 256 butes for WebGPU dynamic buffer alignment
    pub padding: [u8; 236],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct RunInfo {
    pub shot_buffer_entries: u32,
    pub qubit_count: u32,
    pub shot_count: u32,
    pub output_states_per_thread: u32,
    pub threads_per_workgroup: u32,
    pub workgroups: u32,
    pub op_count: u32,
    pub op_index: u32,
    // Ensure alignment on 64 bytes due to first field being u64
    pub padding: [u8; 4],
}

fn write_ops_to_buffer(queue: &wgpu::Queue, buffer: &wgpu::Buffer, ops: &[Op]) {
    // Write the operations to the buffer
    queue.write_buffer(buffer, 0, bytemuck::cast_slice(ops));
}
