#![allow(unused)]

use crate::circuit::Circuit;
use crate::shader_types::Result;

use futures::FutureExt;
use std::num::NonZeroU64;
use wgpu::{
    Adapter, BindGroup, BindGroupLayout, Buffer, ComputePipeline, Device, Queue, ShaderModule,
};

const DO_CAPTURE: bool = true;

pub struct GpuContext {
    device: Device,
    queue: Queue,
    shader_module: ShaderModule,
    bind_group_layout: BindGroupLayout,
    resources: Option<GpuResources>,
}

struct GpuResources {
    pipeline: ComputePipeline,
    state_vector_buffer: Buffer,
    ops_buffer: Buffer,
    results_buffer: Buffer,
    download_buffer: Buffer,
    bind_group: BindGroup,
    circuit: Circuit,
}

impl GpuContext {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter: Adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Failed to create an adapter");

        let downlevel_capabilities = adapter.get_downlevel_capabilities();
        if !downlevel_capabilities
            .flags
            .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
        {
            panic!("Adapter does not support compute shaders");
        }

        let (device, queue): (Device, Queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("failed to create device");

        if DO_CAPTURE {
            unsafe {
                device.start_graphics_debugger_capture();
            }
        }

        // Create the shader module and bind group layout
        let shader_module = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("StateVector bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    // StateVector buffer
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // Ops buffer
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: true,
                        // Specify the per-op slice size so dynamic offsets are allowed
                        min_binding_size: Some(NonZeroU64::new(256).unwrap()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // Result buffer
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        GpuContext {
            device,
            queue,
            shader_module,
            bind_group_layout,
            resources: None,
        }
    }

    pub fn create_resources(&mut self, circuit: Circuit) {
        let state_vector_entries: u64 = 2u64.pow(circuit.qubit_count);
        let result_buffer_size_bytes: u64 = std::mem::size_of::<Result>() as u64 * 100;

        let state_vector_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("StateVector Buffer"),
            size: state_vector_entries * 2 * std::mem::size_of::<f32>() as u64, // 2 floats per complex entry
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        // Initialize ops buffer from the circuit using bytemuck
        let ops_buffer = circuit.create_ops_buffer(&self.device);

        let results_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Results Buffer"),
            size: result_buffer_size_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let download_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Download buffer"),
            size: result_buffer_size_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("StateVector Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: state_vector_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    // Bind a 256-byte slice; dynamic offsets will move this window
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &ops_buffer,
                        offset: 0,
                        size: Some(NonZeroU64::new(256).unwrap()),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: results_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline = self
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("StateVector Pipeline"),
                layout: Some(&self.device.create_pipeline_layout(
                    &wgpu::PipelineLayoutDescriptor {
                        label: Some("StateVector pipeline Layout"),
                        bind_group_layouts: &[&self.bind_group_layout],
                        push_constant_ranges: &[],
                    },
                )),
                module: &self.shader_module,
                entry_point: Some("run_statevector_ops"),
                // When creating the pipeline, override the workgroup size based on the qubit count.
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[("WORKGROUP_SIZE_X", 64.0)], // TODO: Set this based on params
                    ..Default::default()
                },
                cache: None,
            });

        self.resources = Some(GpuResources {
            pipeline,
            state_vector_buffer,
            ops_buffer,
            results_buffer,
            download_buffer,
            bind_group,
            circuit,
        });
    }

    pub async fn run(&self) -> Vec<Result> {
        let resources: &GpuResources = self.resources.as_ref().expect("Resources not initialized");

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("StateVector Command Encoder"),
            });

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("StateVector Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&resources.pipeline);

        let op_count = resources.circuit.ops.len() as u32;
        let workgroup_count: u32 = 10; // TODO: How many workgroups to dispatch based on the qubit count
        for i in 0..op_count {
            let op_offset: u32 = i * 256; // Each op is 256 bytes (aligned)
            compute_pass.set_bind_group(0, &resources.bind_group, &[op_offset]);
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        drop(compute_pass);

        // Copy the results to the download buffer
        encoder.copy_buffer_to_buffer(
            &resources.results_buffer,
            0,
            &resources.download_buffer,
            0,
            resources.download_buffer.size(),
        );

        let command_buffer = encoder.finish();
        self.queue.submit([command_buffer]);

        // Fetching the actual results is a real pain. For details, see:
        // https://github.com/gfx-rs/wgpu/blob/v26/examples/features/src/repeated_compute/mod.rs#L74

        // Cross-platform readback: async map + native poll
        let buffer_slice = resources.download_buffer.slice(..);

        let (sender, receiver) = futures::channel::oneshot::channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |_| {
            sender.send(()).unwrap();
        });

        // On native, drive the GPU and mapping to completion. No-op on the web (where it automatically polls).
        self.device.poll(wgpu::PollType::Wait).unwrap();

        receiver.await.expect("Failed to receive map completion");

        // Read, copy out, and unmap.
        let data = buffer_slice.get_mapped_range();
        let results: Vec<Result> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        resources.download_buffer.unmap();

        results
    }
}

impl Drop for GpuContext {
    fn drop(&mut self) {
        if DO_CAPTURE {
            unsafe {
                self.device.stop_graphics_debugger_capture();
            }
        }
    }
}
