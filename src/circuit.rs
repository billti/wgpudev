#![allow(unused)]

use crate::shader_types::{Op};

// Small helper enum for QIR arg parsing
enum ParsedArg { U32(u32), F32(f32) }

pub struct Circuit {
    pub qubit_count: u32,
    pub ops: Vec<Op>,
}

// ***** The below string parsers largely written by GPT-5 converting from a Python version
// ***** TODO: Definitely some clean-up and improvements possible here

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

        // If the string include ` @__quantum__qis__`, delegate to QIR parsing.
        if src.contains("@__quantum__qis__") {
            return Self::from_qir_str(src);
        }

        use crate::shader_types::ops;

        let mut ops_vec: Vec<Op> = Vec::new();
        let mut max_qubit: i64 = -1;

        // Add a reset op at the start to signal the start of a circuit.
        ops_vec.push(Op {
            op_idx: 0, // This is the first op, so index is 0
            op_id: ops::RESET,
            q1: 0,
            q2: 0,
            q3: 0,
            angle: 0.0,
            padding: [0; 232],
        });

        let mut op_idx = 1;

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

            let op = Op {
                op_idx,
                op_id,
                q1,
                q2,
                q3,
                angle: angle.unwrap_or(0.0),
                padding: [0; 232],
            };
            ops_vec.push(op);
            op_idx += 1;
        }
        ops_vec.push(Op {
            op_idx: op_idx as u32,
            op_id: ops::MEVERYZ, // Implicit measurement at the end of the circuit
            q1: 0,
            q2: 0,
            q3: 0,
            angle: 0.0,
            padding: [0; 232],
        });

        let qubit_count = if max_qubit >= 0 { (max_qubit as u32) + 1 } else { 0 };
        Ok(Circuit { qubit_count, ops: ops_vec })
    }

    /// Parse a QIR (LLVM IR text) program and build a Circuit.
    /// Only a minimal subset of QIR is supported: selected QIS gates (sx, x, y, z, h, s, t, s_adj, t_adj,
    /// rx, ry, rz, cz, cx, rzz, ccx, m) and RT calls related to initialization/output are ignored.
    /// The QIR must declare attributes #0 including base_profile and required_num_qubits/results, and the
    /// entry point must be `define void @...() #0 { ... }`.
    pub fn from_qir_str(qir: &str) -> Result<Self, String> {
        use crate::shader_types::ops;

        // Helper: extract the attribute block for #0
        let attr_idx = qir.find("attributes #0 = {").ok_or_else(|| {
            "QIR does not contain the required attributes (#0)".to_string()
        })?;
        let after_attr = &qir[attr_idx + "attributes #0 = {".len()..];
        let end_brace_rel = after_attr.find('}').ok_or_else(|| {
            "QIR attributes #0 block not terminated with '}'".to_string()
        })?;
        let attr_block = &after_attr[..end_brace_rel];

        // Tiny parser for key values like "qir_profiles"="base_profile"
        fn find_attr_value<'a>(s: &'a str, key: &str) -> Option<&'a str> {
            let key_pat = format!("\"{}\"=\"", key);
            let start = s.find(&key_pat)? + key_pat.len();
            let rest = &s[start..];
            let end = rest.find('"')?;
            Some(&rest[..end])
        }

        let profile = find_attr_value(attr_block, "qir_profiles")
            .ok_or_else(|| "QIR does not contain qir_profiles attribute in #0".to_string())?;
        if profile != "base_profile" {
            return Err(format!("Profile is not base_profile: {}", profile));
        }

        let qubits_str = find_attr_value(attr_block, "required_num_qubits")
            .ok_or_else(|| "QIR missing required_num_qubits in attributes #0".to_string())?;
        let results_str = find_attr_value(attr_block, "required_num_results")
            .ok_or_else(|| "QIR missing required_num_results in attributes #0".to_string())?;
        let declared_qubits: u32 = qubits_str
            .parse::<u32>()
            .map_err(|_| format!("Invalid required_num_qubits: {}", qubits_str))?;
        let declared_results: u32 = results_str
            .parse::<u32>()
            .map_err(|_| format!("Invalid required_num_results: {}", results_str))?;

        // Find the entry point function body: a line with "define void @...() #0" followed by '{'
        let mut in_entry = false;
        let mut ops_vec: Vec<Op> = Vec::new();
        let mut max_qubit: i64 = -1;
        let mut op_idx: u32 = 0;
        let mut saw_measure = false;

        // Always start with a RESET sentinel op
        ops_vec.push(Op { op_idx, op_id: ops::RESET, q1: 0, q2: 0, q3: 0, angle: 0.0, padding: [0; 232] });
        op_idx += 1;

        for raw in qir.lines() {
            let line = raw.trim();
            if line.is_empty() { continue; }

            if !in_entry {
                // Detect start of entry point
                if line.starts_with("define void @") && line.contains("()") && line.contains(" #0") {
                    // It can have the opening brace on this or the next line; if not, we'll flip in_entry now
                    in_entry = true;
                }
                continue;
            }

            // End of entry function
            if line.starts_with("ret ") || line == "}" { in_entry = false; break; }

            // We're only interested in QIS/RT calls
            if !line.contains("call ") || !line.contains("@__quantum__") { continue; }

            // Parse call like: call void @__quantum__qis__rz__body(double 0.5, %Qubit* inttoptr (i64 1 to %Qubit*))
            let at_pos = if let Some(p) = line.find("@__quantum__") { p } else { continue };
            let after = &line[at_pos + "@__quantum__".len()..];
            let cat_end = after.find("__").ok_or_else(|| format!("Invalid QIR line: {}", line))?;
            let category = &after[..cat_end]; // "qis" | "rt"
            let after_cat = &after[cat_end + 2..];

            // Extract name up to '(' (strip optional __body suffix)
            let paren_idx = after_cat.find('(').ok_or_else(|| format!("Invalid QIR call (no paren): {}", line))?;
            let mut name = after_cat[..paren_idx].to_string();
            if let Some(pos) = name.rfind("__body") { if pos + 6 == name.len() { name.truncate(pos); } }
            // If the name accidentally ends with trailing underscores (rare), trim only if it ends with "__" explicitly
            if name.ends_with("__") { name.truncate(name.len().saturating_sub(2)); }

            // Extract argument substring between the first '(' after name and the last ')'
            let args_start_global = at_pos + "@__quantum__".len() + cat_end + 2 + paren_idx;
            // Find the last ')' in line as close approximation (arguments don't contain commas outside inner parens)
            let last_paren = line.rfind(')').ok_or_else(|| format!("Invalid QIR call (no closing paren): {}", line))?;
            let args_str = &line[args_start_global + 1..last_paren];

            // Split top-level args by comma
            let mut parsed_nums: Vec<Option<ParsedArg>> = Vec::new();
            for part in args_str.split(',') {
                let arg = part.trim();
                if arg.is_empty() { continue; }
                if arg.starts_with("%Qubit* inttoptr (i64 ") || arg.starts_with("%Result* inttoptr (i64 ") {
                    // extract number after "(i64 "
                    if let Some(pos) = arg.find("(i64 ") {
                        let rest = &arg[pos + 5..];
                        // up to space
                        let mut digits = String::new();
                        for ch in rest.chars() { if ch.is_ascii_digit() { digits.push(ch); } else { break; } }
                        if digits.is_empty() { return Err(format!("Invalid QIR argument: {}", arg)); }
                        let val: u32 = digits.parse().map_err(|_| format!("Invalid QIR argument: {}", arg))?;
                        parsed_nums.push(Some(ParsedArg::U32(val)));
                    } else {
                        return Err(format!("Invalid QIR argument: {}", arg));
                    }
                } else if arg.starts_with("double ") {
                    let v = arg[7..].trim();
                    let angle: f32 = v.parse::<f32>().map_err(|_| format!("Invalid QIR argument: {}", arg))?;
                    parsed_nums.push(Some(ParsedArg::F32(angle)));
                } else if arg.starts_with("i64 ") {
                    let v = arg[4..].trim();
                    let val: u32 = v.parse::<u32>().map_err(|_| format!("Invalid QIR argument: {}", arg))?;
                    parsed_nums.push(Some(ParsedArg::U32(val)));
                } else if arg == "i8* null" {
                    parsed_nums.push(None);
                } else {
                    return Err(format!("Invalid QIR argument: {}", arg));
                }
            }

            // Map operation
            if category == "rt" {
                // Ignore runtime bookkeeping in base profile
                continue;
            }

            let op_id = match name.as_str() {
                "id" => ops::ID,
                "x" => ops::X,
                "y" => ops::Y,
                "z" => ops::Z,
                "h" => ops::H,
                "s" => ops::S,
                "s_adj" | "s__adj" => ops::S_ADJ,
                "t" => ops::T,
                "t_adj" | "t__adj" => ops::T_ADJ,
                "sx" => ops::SX,
                "sx_adj" | "sx__adj" => ops::SX_ADJ,
                "rx" => ops::RX,
                "ry" => ops::RY,
                "rz" => ops::RZ,
                "cx" => ops::CX,
                "cz" => ops::CZ,
                "rzz" => ops::RZZ,
                "ccx" => ops::CCX,
                "m" => { saw_measure = true; ops::MZ }, // normalize m -> mz
                other => return Err(format!("Unsupported QIR QIS op: {}", other)),
            };

            // Build op fields from parsed args
            let mut angle_val: f32 = 0.0;
            let mut q1: u32 = 0; let mut q2: u32 = 0; let mut q3: u32 = 0;

            match op_id {
                ops::RX | ops::RY | ops::RZ => {
                    // Expect [angle, qubit]
                    if parsed_nums.len() < 2 { return Err(format!("{} expects angle and qubit", name)); }
                    angle_val = match parsed_nums[0] { Some(ParsedArg::F32(a)) => a, _ => return Err(format!("{} first arg must be double", name)) };
                    q1 = match parsed_nums[1] { Some(ParsedArg::U32(n)) => n, _ => return Err(format!("{} second arg must be qubit", name)) };
                }
                ops::RZZ | ops::CX | ops::CZ => {
                    // Expect two qubits (angles only for RZZ handled separately if present before)
                    if op_id == ops::RZZ {
                        if parsed_nums.len() < 3 { return Err("rzz expects angle and two qubits".to_string()); }
                        angle_val = match parsed_nums[0] { Some(ParsedArg::F32(a)) => a, _ => return Err("rzz first arg must be double".to_string()) };
                        q1 = match parsed_nums[1] { Some(ParsedArg::U32(n)) => n, _ => return Err("rzz second arg must be qubit".to_string()) };
                        q2 = match parsed_nums[2] { Some(ParsedArg::U32(n)) => n, _ => return Err("rzz third arg must be qubit".to_string()) };
                    } else {
                        if parsed_nums.len() < 2 { return Err(format!("{} expects two qubits", name)); }
                        q1 = match parsed_nums[0] { Some(ParsedArg::U32(n)) => n, _ => return Err(format!("{} first arg must be qubit", name)) };
                        q2 = match parsed_nums[1] { Some(ParsedArg::U32(n)) => n, _ => return Err(format!("{} second arg must be qubit", name)) };
                    }
                }
                ops::CCX => {
                    if parsed_nums.len() < 3 { return Err("ccx expects three qubits".to_string()); }
                    q1 = match parsed_nums[0] { Some(ParsedArg::U32(n)) => n, _ => return Err("ccx first arg must be qubit".to_string()) };
                    q2 = match parsed_nums[1] { Some(ParsedArg::U32(n)) => n, _ => return Err("ccx second arg must be qubit".to_string()) };
                    q3 = match parsed_nums[2] { Some(ParsedArg::U32(n)) => n, _ => return Err("ccx third arg must be qubit".to_string()) };
                }
                ops::MZ => {
                    // m(%Qubit*, %Result*)
                    if parsed_nums.is_empty() { return Err("m expects qubit".to_string()); }
                    q1 = match parsed_nums[0] { Some(ParsedArg::U32(n)) => n, _ => return Err("m first arg must be qubit".to_string()) };
                }
                _ => {
                    // Single-qubit ops: take first qubit
                    if parsed_nums.is_empty() { return Err(format!("{} expects qubit", name)); }
                    q1 = match parsed_nums[0] { Some(ParsedArg::U32(n)) => n, _ => return Err(format!("{} first arg must be qubit", name)) };
                }
            }

            max_qubit = max_qubit.max(q1 as i64);
            max_qubit = max_qubit.max(q2 as i64);
            max_qubit = max_qubit.max(q3 as i64);

            ops_vec.push(Op { op_idx, op_id, q1, q2, q3, angle: angle_val, padding: [0; 232] });
            op_idx += 1;
        }

        if in_entry {
            return Err("QIR entry point not properly terminated".to_string());
        }

        // If no explicit measurements were found, add an implicit measure-every-z at end
        ops_vec.push(Op { op_idx, op_id: ops::MEVERYZ, q1: 0, q2: 0, q3: 0, angle: 0.0, padding: [0; 232] });

        // Determine qubit count from declared and observed
        let inferred_qubits = if max_qubit >= 0 { (max_qubit as u32) + 1 } else { 0 };
        let qubit_count = declared_qubits.max(inferred_qubits);
        // declared_results currently unused; could be used in the future for output shape validation
        let _ = declared_results;

        Ok(Circuit { qubit_count, ops: ops_vec })
    }

    pub fn create_ops_buffer(&self, device: &wgpu::Device, xcode_traceable: bool) -> wgpu::Buffer {
        // This safely treats &[CircuitOp] as &[u8] due to Pod + repr(C)
        use wgpu::util::DeviceExt;
        if xcode_traceable {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Ops Buffer"),
                size: (self.ops.len() * std::mem::size_of::<Op>()) as u64,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::MAP_WRITE
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: true,
            });

            buffer
                .slice(..)
                .get_mapped_range_mut()
                .copy_from_slice(bytemuck::cast_slice(&self.ops));

            buffer.unmap();
            buffer
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Ops Buffer"),
                contents: bytemuck::cast_slice(&self.ops),
                usage: wgpu::BufferUsages::STORAGE,
            })
        }
    }
}
