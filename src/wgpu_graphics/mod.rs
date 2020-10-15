use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use anyhow::{Error, Result};
use legion::{system, World, Resources};

mod mesh;

struct Pass {
    pipeline: wgpu::RenderPipeline,
    bind_grou: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
}

struct Renderer {
    forward_pass: Pass,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
}

pub fn start() -> Result<()> {
    //futures::executor::block_on(run(event_loop, window, wgpu::TextureFormat::Bgra8UnormSrgb));
    Ok(())
}

pub async fn setup(world: &mut World, resources: &mut Resources) -> Result<()> {
    let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop)?;

    let backend = if let Ok(backend) = std::env::var("WGPU_BACKEND") {
        match backend.to_lowercase().as_str() {
            "vulkan" => wgpu::BackendBit::VULKAN,
            "metal" => wgpu::BackendBit::METAL,
            "dx12" => wgpu::BackendBit::DX12,
            "dx11" => wgpu::BackendBit::DX11,
            "gl" => wgpu::BackendBit::GL,
            "webgpu" => wgpu::BackendBit::BROWSER_WEBGPU,
            other => panic!("Unknown backend: {}", other),
        }
    } else { wgpu::BackendBit::PRIMARY };

    let instance = wgpu::Instance::new(backend);
    let size = window.inner_size();
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            //power_preference: wgpu::PowerPreference::HighPerformance,
            power_preference: wgpu::PowerPreference::default(),
            // Request an adapter which can render to a surface
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();

    // Optional trace file
    let trace_dir = std::env::var("WGPU_TRACE");
    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            }, 
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .unwrap();

    // Load shaders from disk
    let vs_module = device.create_shader_module(wgpu::include_spirv!("shader.vert.spv"));
    let fs_module = device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"));

    let mut swap_chain_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        // Wait for vsync, but do not cap framerate
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

    let vertex_size = std::mem::size_of::<mesh::Vertex>();
    let (cube_vertex_data, cube_index_data) = mesh::create_cube();
    use wgpu::util::DeviceExt;
    let cube_vertex_buf = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(&cube_vertex_data),
            usage: wgpu::BufferUsage::VERTEX,
        }
    );
    let cube_index_buf = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(&cube_index_data),
            usage: wgpu::BufferUsage::INDEX,
        }
    );
    let (plane_vertex_data, plane_index_data) = mesh::create_plane(7);
    let plane_vertex_buf = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Plane Vertex Buffer"),
            contents: bytemuck::cast_slice(&plane_vertex_data),
            usage: wgpu::BufferUsage::VERTEX
        }
    );
    let plane_index_buf = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Plane Index Buffer"),
            contents: bytemuck::cast_slice(&plane_index_data),
            usage: wgpu::BufferUsage::INDEX
        }
    );

    let mesh_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            }
        ],
    });

    let global_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<mesh::GlobalUniforms>() as wgpu::BufferAddress,
                    ),
                },
                count: None
            }
        ]
    });

    let global_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Global uniform buffer"),
        size: std::mem::size_of::<mesh::GlobalUniforms>() as wgpu::BufferAddress,
        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        mapped_at_creation: false,
    });

    let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &global_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(global_uniform_buf.slice(..)),
            },
        ],
    });

    let model_uniform_size = std::mem::size_of::<mesh::ModelUniforms>() as wgpu::BufferAddress;
    let model_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 0,
        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        mapped_at_creation: false,
    });

    // MeshPipeline (untextured)
    //let part_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    //    label: None,
    //    entries: &[
    //        // Material factors
    //        wgpu::BindGroupLayoutEntry {
    //            binding: 0,
    //            visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
    //            ty: wgpu::BindingType::UniformBuffer {
    //                dynamic: false,
    //                min_binding_size: wgpu::BufferSi
    //            }
    //        }
    //    ]
    //})

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Main render pipeline layout"),
        bind_group_layouts: &[
            &global_bind_group_layout,
            &mesh_bind_group_layout,
        ],
        push_constant_ranges: &[]
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Main render pipeline"),
        layout: Some(&pipeline_layout),
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        //rasterization_state: Some(wgpu::RasterizationStateDescriptor {
        //    front_face: wgpu::FrontFace::Ccw,
        //    //cull_mode: wgpu::CullMode::Back,
        //    cull_mode: wgpu::CullMode::None,
        //    depth_bias: 0,
        //    depth_bias_slope_scale: 0.0,
        //    depth_bias_clamp: 0.0,
        //    clamp_depth: false,
        //}),
        rasterization_state: None,
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[
            // TODO: Why is this repeated?
            wgpu::ColorStateDescriptor {
                format: swap_chain_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            },
            wgpu::ColorStateDescriptor {
                format: swap_chain_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            },
        ],
        //depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
        //    format: wgpu::TextureFormat::Depth32Float,
        //    depth_write_enabled: true,
        //    depth_compare: wgpu::CompareFunction::Less,
        //    stencil: wgpu::StencilStateDescriptor {
        //        front: wgpu::StencilStateFaceDescriptor::IGNORE,
        //        back: wgpu::StencilStateFaceDescriptor::IGNORE,
        //        read_mask: 0,
        //        write_mask: 0,
        //    },
        //}),
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: std::mem::size_of::<mesh::Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: 
                    // TODO
                    &wgpu::vertex_attr_array![
                        // Position
                        0 => Float3,
                        // Normal
                        1 => Float3,
                    ],
            }],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });


    Ok(())
}

/*
async fn run(event_loop: EventLoop<()>, window: Window, swapchain_format: wgpu::TextureFormat) {

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // event_loop.run never returns, so we must do this to ensure 
        // the resources are properly cleaned up.
        // By moving all of those resources to an empty variable, all of them get dropped
        // and their drop() functions get called.
        let _ = (&instance, &adapter, &vs_module, &fs_module, &pipeline_layout);

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                // Recreate the swap chain with the new size
                swap_chain_desc.width = size.width;
                swap_chain_desc.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { .. } => {},
                WindowEvent::CursorMoved { .. } => {},
                _ => {},
            },
            // Emmited when all of the event loop's input events have been processed and redraw processing is about to begin.
            // Normally, we would use Event::RedrawRequested for rendering, but we can also just render here, because it's a game
            // that has to render continuously either way.
            Event::MainEventsCleared => {

            },
            Event::RedrawRequested(_) => {
                let frame = swap_chain
                    .get_current_frame()
                    .map_err(|err| 
                        Error::msg("Failed to acquire next swap chain texture")
                            .context(err)
                    )
                    .unwrap()
                    .output;

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });                    
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                // Clear the framebuffer with a color
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });
                    // Draw with our pipeline
                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.draw(0..3, 0..1);
                }

                queue.submit(Some(encoder.finish()));
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {},
        }
    });
}
*/