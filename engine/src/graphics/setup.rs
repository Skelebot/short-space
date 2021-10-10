use std::rc::Rc;

use eyre::{
    eyre::eyre,
    eyre::{ContextCompat, WrapErr},
    Result,
};
use legion::{Resources, World};
use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::{
    graphics::{
        debug::DebugPass,
        mesh::{MeshPass, RenderMeshLayouts},
    },
    state::CustomEvent,
};

use crate::graphics::*;

pub async fn setup(
    world: &mut World,
    resources: &mut Resources,
) -> Result<(Graphics, EventLoop<CustomEvent>)> {
    let event_loop = EventLoop::<CustomEvent>::with_user_event();
    let window = WindowBuilder::new()
        .with_title("Endless Josh")
        .build(&event_loop)?;

    let backend: Option<wgpu::Backends> = if let Ok(backend) = std::env::var("WGPU_BACKEND") {
        Some(
            match backend.to_lowercase().as_str() {
                "vulkan" => wgpu::Backend::Vulkan,
                "metal" => wgpu::Backend::Metal,
                "dx12" => wgpu::Backend::Dx12,
                "dx11" => wgpu::Backend::Dx11,
                "gl" => wgpu::Backend::Gl,
                "webgpu" => wgpu::Backend::BrowserWebGpu,
                other => panic!("Unknown backend: {}", other),
            }
            .into(),
        )
    } else {
        None
    };

    // ??
    use wgpu::Backends;
    let instance = wgpu::Instance::new(backend.unwrap_or_else(Backends::all));
    let size = window.inner_size();
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            //power_preference: wgpu::PowerPreference::HighPerformance,
            power_preference: wgpu::PowerPreference::default(),
            // Request an adapter which can render to a surface
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .ok_or_else(|| eyre!("Couldn't find a compatible graphics adapter for backend: {:?}\nIf you want to force a different backend, set the WGPU_BACKEND environmental variable.\nKeep in mind that OpenGL is not currently supported.", backend))?;

    // Optional trace file
    let trace_dir = std::env::var("WGPU_TRACE");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .wrap_err_with(|| "Failed to create the graphics device")?;

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface
            .get_preferred_format(&adapter)
            .wrap_err_with(|| "Failed to get preferred format")?,
        width: size.width,
        height: size.height,
        //present_mode: wgpu::PresentMode::Immediate,
        // Wait for vsync, but do not cap framerate
        //present_mode: wgpu::PresentMode::Mailbox,
        // Wait for vsync AND cap framerate
        present_mode: wgpu::PresentMode::Fifo,
    };

    surface.configure(&device, &surface_config);

    // Initialize render passes
    let mesh_pass = MeshPass::new(&device, &surface_config, world, resources)?;
    //let ui_pass = UiPass::new(&device, &surface_config, &window, &queue, world, resources)?;
    let debug_pass = DebugPass::new(&device, &surface_config, &window, &queue, world, resources)?;

    let device = Rc::new(device);
    let queue = Rc::new(queue);
    let window = Rc::new(window);

    // Insert related resources
    resources.insert(event_loop.create_proxy());
    let shared = GraphicsShared {
        device: device.clone(),
        queue: queue.clone(),
        window: window.clone(),
        // TODO: Do something about those layouts
        mesh_layouts: RenderMeshLayouts {
            mesh: mesh_pass.mesh_bind_group_layout.clone(),
            material: mesh_pass.pipelines.clone(),
        },
    };
    resources.insert(shared.clone());

    // Depth testing
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth texture"),
        size: wgpu::Extent3d {
            width: surface_config.width,
            height: surface_config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    });

    let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    Ok((
        Graphics {
            device,
            queue,
            window,
            mesh_pass,
            //ui_pass,
            debug_pass: Some(debug_pass),
            surface_config,
            surface,
            shared,
            depth_texture,
            depth_texture_view,
        },
        event_loop,
    ))
}
