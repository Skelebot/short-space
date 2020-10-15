enum GameType {
    FFA,
    TOURNAMENT,
    SINGLE_PLAYER,
    TEAM,
    CTF,
    ONEFCTF,
    OBELISK,
}

    // Create the pipeline layout
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    // Create the render pipeline
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        // Vertex stage, where we execute our vertex shader
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        // Fragment stage
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        // Use the default rasterizer state: no culling, no depth bias
        // TODO: Enable backface culling and other optimizations
        rasterization_state: None,
        // Everything in our pipeline will be a list of triangles (like meshes)
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[swapchain_format.into()],
        // We don't use depth stenciling (for now)
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
        },
        // Number of samples per pixel (for MSAA)
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });