#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct CircuitOp {
    pub op_id: u32, // op type
    pub q1: u32, // target qubit
    pub q2: u32, // control qubit for cx, cz, etc., or second qubit for rzz
    pub q3: u32, // additional control for ccx
    pub arg: f32, // rx, ry, rz, rzz
    // Pad out to 256 bytes for buffer alignment (WebGPU requirement for dynamic offset buffers)
    pub padding: [u8; 236],
}

pub struct Circuit {
    pub qubit_count: u32,
    pub ops: Vec<CircuitOp>,
}

// Add a helper to create the ops buffer using bytemuck to cast the slice.
impl Circuit {
    pub fn create_ops_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        // This safely treats &[CircuitOp] as &[u8] due to Pod + repr(C)
        use wgpu::util::DeviceExt;
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ops Buffer"),
            contents: bytemuck::cast_slice(&self.ops),
            usage: wgpu::BufferUsages::STORAGE,
        })
    }
}
