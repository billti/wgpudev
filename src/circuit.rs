#![allow(unused)]

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
    /// Parse a circuit description from a string. Each non-empty line describes one op.
    /// Syntax examples:
    ///   "x 0"
    ///   "rz (0.5) 1"
    ///   "rx (1.234) 2"
    ///   "cz 0 1"
    ///   "rzz (0.125) 1 3"
    ///   "ccx 0 1 2"
    /// Angle may also be attached to the op token, e.g. "rz(0.5) 1".
    pub fn from_str(src: &str) -> Result<Self, String> {
        use crate::shader_types::ops;

        let mut ops_vec: Vec<CircuitOp> = Vec::new();
        let mut max_qubit: i64 = -1;

        for (lineno, raw_line) in src.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() { continue; }
            // Allow comments starting with '#'
            if let Some(idx) = line.find('#') { if idx == 0 { continue; } }

            let mut parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() { continue; }

            // Extract op name and optional inline angle like "rz(0.5)"
            let mut name_token = parts[0].to_ascii_lowercase();
            let mut angle: Option<f32> = None;

            if let Some(open) = name_token.find('(') {
                let close = name_token.rfind(')')
                    .ok_or_else(|| format!("Line {}: invalid angle token: {}", lineno + 1, parts[0]))?;
                let inner = &name_token[open + 1..close];
                angle = Some(inner.parse::<f32>().map_err(|_| format!(
                    "Line {}: invalid angle value: {}", lineno + 1, inner
                ))?);
                name_token.truncate(open);
                // Replace the original part with the cleaned name (no angle)
                parts[0] = &line[..line.find(parts[0]).unwrap_or(0)]; // placeholder; we won't use parts[0] again
            }

            let op_id = match name_token.as_str() {
                "id" => ops::ID,
                "reset" => ops::RESET,
                "x" => ops::X,
                "y" => ops::Y,
                "z" => ops::Z,
                "h" => ops::H,
                "s" => ops::S,
                "s_adj" | "sadj" | "sdag" => ops::S_ADJ,
                "t" => ops::T,
                "t_adj" | "tadj" | "tdag" => ops::T_ADJ,
                "sx" => ops::SX,
                "sx_adj" | "sxadj" => ops::SX_ADJ,
                "rx" => ops::RX,
                "ry" => ops::RY,
                "rz" => ops::RZ,
                "cx" => ops::CX,
                "cz" => ops::CZ,
                "rzz" => ops::RZZ,
                "ccx" | "toffoli" => ops::CCX,
                "mz" => ops::MZ,
                "mresetz" => ops::MRESETZ,
                other => return Err(format!("Line {}: invalid operation: {}", lineno + 1, other)),
            };

            // Determine if an angle is required or forbidden and figure out where qubit indices start.
            let mut qubit_start_index = 1usize; // name at index 0
            let angle_required = matches!(op_id, ops::RX | ops::RY | ops::RZ | ops::RZZ);
            let angle_allowed = angle_required; // currently only these ops accept an angle

            if angle_required && angle.is_none() {
                // Accept a space-separated angle token like "(0.5)"
                if parts.len() > 1 && parts[1].starts_with('(') {
                    let token = parts[1];
                    if !token.ends_with(')') {
                        return Err(format!("Line {}: malformed angle token: {}", lineno + 1, token));
                    }
                    let inner = &token[1..token.len() - 1];
                    angle = Some(inner.parse::<f32>().map_err(|_| format!(
                        "Line {}: invalid angle value: {}", lineno + 1, inner
                    ))?);
                    qubit_start_index = 2; // name + angle
                } else {
                    return Err(format!(
                        "Line {}: {} requires an angle argument in parentheses, e.g. {} (0.5) ...",
                        lineno + 1,
                        name_token,
                        name_token
                    ));
                }
            } else if !angle_allowed && angle.is_some() {
                return Err(format!("Line {}: {} does not take an angle", lineno + 1, name_token));
            }

            // Count required qubits
            let required_qubits: usize = match op_id {
                ops::CX | ops::CZ => 2,
                ops::RZZ => 2,
                ops::CCX => 3,
                _ => 1,
            };

            let have_qubits = parts.len().saturating_sub(qubit_start_index);
            if have_qubits != required_qubits {
                return Err(format!(
                    "Line {}: invalid argument count for '{}' (got {}, expected {})",
                    lineno + 1,
                    line,
                    have_qubits + if angle.is_some() { 1 } else { 0 },
                    required_qubits + if angle_required { 1 } else { 0 }
                ));
            }

            // Parse qubit indices
            let parse_qubit = |tok: &str| -> Result<u32, String> {
                tok.parse::<u32>()
                    .map_err(|_| format!("Line {}: invalid qubit index: {}", lineno + 1, tok))
            };

            let mut q1 = 0u32;
            let mut q2 = 0u32;
            let mut q3 = 0u32;

            if required_qubits >= 1 {
                q1 = parse_qubit(parts[qubit_start_index])?;
                max_qubit = max_qubit.max(q1 as i64);
            }
            if required_qubits >= 2 {
                q2 = parse_qubit(parts[qubit_start_index + 1])?;
                max_qubit = max_qubit.max(q2 as i64);
            }
            if required_qubits >= 3 {
                q3 = parse_qubit(parts[qubit_start_index + 2])?;
                max_qubit = max_qubit.max(q3 as i64);
            }

            let op = CircuitOp {
                op_id,
                q1,
                q2,
                q3,
                arg: angle.unwrap_or(0.0),
                padding: [0; 236],
            };
            ops_vec.push(op);
        }

        let qubit_count = if max_qubit >= 0 { (max_qubit as u32) + 1 } else { 0 };
        Ok(Circuit { qubit_count, ops: ops_vec })
    }

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
