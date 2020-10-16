use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use anyhow::{Error, Result};
use legion::{system, World, Resources};

mod mesh;

pub fn start() -> Result<()> {
    futures::executor::block_on(setup());
    Ok(())
}

pub async fn setup() -> Result<()> {
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

    let aspect = size.width as f32 / size.height as f32;
    //let mut camera = crate::graphics::Camera::new(aspect, 3.14/2.0, 0.01, 1000.0); 
    let mut camera = crate::graphics::Camera::new(aspect, 45.0, 0.1, 20.0); 
    //camera.position.translation.vector.x += 3.0;
    camera.position.translation.vector.y += -5.0;
    camera.position.translation.vector.z += 6.0;
    camera.position.rotation = na::UnitQuaternion::from_axis_angle(
        &na::Vector::x_axis(), 
        -60.0f32.to_radians()
    );

    let global_uniforms = mesh::GlobalUniforms {
        view_proj: camera.get_vp_matrix().into(),
        camera_pos: camera.position.translation.vector.into(),
    };

    use wgpu::util::DeviceExt;
    let global_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Global uniform buffer"),
        contents: bytemuck::bytes_of(&global_uniforms),
        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
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
        color_states: &[swap_chain_desc.format.into()],
        //    // TODO: Why is this repeated?
        //    wgpu::ColorStateDescriptor {
        //        format: swap_chain_desc.format,
        //        color_blend: wgpu::BlendDescriptor::REPLACE,
        //        alpha_blend: wgpu::BlendDescriptor::REPLACE,
        //        write_mask: wgpu::ColorWrite::ALL,
        //    },
        //    wgpu::ColorStateDescriptor {
        //        format: swap_chain_desc.format,
        //        color_blend: wgpu::BlendDescriptor::REPLACE,
        //        alpha_blend: wgpu::BlendDescriptor::REPLACE,
        //        write_mask: wgpu::ColorWrite::ALL,
        //    },
        //],
        //depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
        //    format: wgpu::TextureFormat::Depth32Float,
        //    depth_write_enabled: true,
        //    depth_compare: wgpu::CompareFunction::Less,
        //    stencil: wgpu::StencilStateDescriptor::default(),
        //}),
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: std::mem::size_of::<mesh::Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: 
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

    let mesh_pass = mesh::MeshPass {
        pipeline: pipeline,
        mesh_bind_group_layout,
        global_bind_group_layout,
        global_bind_group: global_bind_group,
        global_uniform_buf: global_uniform_buf,
    };

    let _ = (&instance, &adapter, &vs_module, &fs_module, &pipeline_layout);

    run(device, swap_chain, swap_chain_desc, surface, event_loop, window, mesh_pass, swapchain_format, queue, camera).await;
    Ok(())
}

async fn run(
    mut device: wgpu::Device,
    mut swap_chain: wgpu::SwapChain,
    mut swap_chain_desc: wgpu::SwapChainDescriptor,
    surface: wgpu::Surface,
    event_loop: EventLoop<()>,
    window: Window,
    mesh_pass: mesh::MeshPass,
    swapchain_format: wgpu::TextureFormat,
    queue: wgpu::Queue,
    mut camera: crate::graphics::Camera,
) {
    debug!("Creating cube data");
    let cube_model = mesh::Model::from_data(mesh::create_cube(), &mut device, &mesh_pass);
    let plane_model = mesh::Model::from_data(mesh::create_plane(10), &mut device, &mesh_pass);

    let model_uniform_size = std::mem::size_of::<mesh::ModelUniforms>() as wgpu::BufferAddress;

    debug!("Running the event loop");
    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // event_loop.run never returns, so we must do this to ensure 
        // the resources are properly cleaned up.
        // By moving all of those resources to an empty variable, all of them get dropped
        // and their drop() functions get called.

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                // Recreate the swap chain with the new size
                swap_chain_desc.width = size.width;
                swap_chain_desc.height = size.height;
                camera.update_aspect(size.width as f32/size.height as f32);
                let proj_view: [[f32; 4]; 4] = camera.get_vp_matrix().into();
                queue.write_buffer(
                    &mesh_pass.global_uniform_buf,
                    0,
                    // FIXME: cast_slice()?
                    bytemuck::bytes_of(&proj_view),
                );
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
                window.request_redraw();
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
                    let correction = na::Matrix4::new(
                        1.0, 0.0, 0.0, 0.0,
                        0.0, 1.0, 0.0, 0.0,
                        0.0, 0.0, 0.5, 0.0,
                        0.0, 0.0, 0.5, 1.0,
                    );
                    // Upload global uniforms
                    let view_proj = correction * camera.get_vp_matrix();
                    let global_uniforms = mesh::GlobalUniforms {
                        view_proj: view_proj.into(),
                        camera_pos: na::Vector3::new(0.0, 0.0, 0.0).into(),
                    };
                    queue.write_buffer(
                        &mesh_pass.global_uniform_buf,
                        0,
                        bytemuck::bytes_of(&global_uniforms)
                    );
                    // Draw with our pipeline
                    // Per pass
                    render_pass.set_pipeline(&mesh_pass.pipeline);
                    render_pass.set_bind_group(0, &mesh_pass.global_bind_group, &[]);
                    // Per entity
                    // CUBE
                    // Upload mesh transform matrices
                    //let transform: [[f32; 4]; 4] = cube_model.world.into();
                    //queue.write_buffer(
                    //    &cube_model.uniform_buf,
                    //    0,
                    //    bytemuck::bytes_of(&transform)
                    //);
                    //render_pass.set_bind_group(1, &cube_model.bind_group, &[]);
                    //// pass.set_bind_group(1, entity_bind_group, &[entity.uniform_offset])
                    //render_pass.set_index_buffer(cube_model.index_buf.slice(..));
                    //render_pass.set_vertex_buffer(0, cube_model.vertex_buf.slice(..));
                    //render_pass.draw_indexed(
                    //    0 .. cube_model.index_count as u32,
                    //    0,
                    //    0..1,
                    //);
                    // PLANE
                    let transform: [[f32; 4]; 4] = plane_model.world.into();
                    queue.write_buffer(
                        &plane_model.uniform_buf,
                        0,
                        bytemuck::bytes_of(&transform)
                    );
                    render_pass.set_bind_group(1, &plane_model.bind_group, &[]);
                    // pass.set_bind_group(1, entity_bind_group, &[entity.uniform_offset])
                    render_pass.set_index_buffer(plane_model.index_buf.slice(..));
                    render_pass.set_vertex_buffer(0, plane_model.vertex_buf.slice(..));
                    render_pass.draw_indexed(
                        0 .. plane_model.index_count as u32,
                        0,
                        0..1,
                    );

                }

                queue.submit(Some(encoder.finish()));
            },
            _ => {},
        }
    });
}