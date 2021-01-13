use std::rc::Rc;

use eyre::{eyre::eyre, eyre::WrapErr, Result};
use legion::{Resources, World};
use winit::{event_loop::EventLoop, window::Window};

use crate::state::CustomEvent;

use super::{mesh_pass::MeshPass, Graphics};

pub async fn setup(
    world: &mut World,
    resources: &mut Resources,
) -> Result<(Graphics, EventLoop<CustomEvent>)> {
    let event_loop = EventLoop::<CustomEvent>::with_user_event();
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
    } else {
        wgpu::BackendBit::PRIMARY
    };

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
        .ok_or_else(|| eyre!("Couldn't find a compatible graphics adapter for backend: {:?}\nIf you want to force a different backend, set the WGPU_BACKEND environmental variable.\nKeep in mind that OpenGL is not currently supported.", backend))?;

    // Optional trace file
    let trace_dir = std::env::var("WGPU_TRACE");

    // Create the logical device and command queue
    let (mut device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .wrap_err_with(|| "Failed to create the graphics device")?;

    let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let swap_chain_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        // Wait for vsync, but do not cap framerate
        //present_mode: wgpu::PresentMode::Mailbox,
        // Wait for vsync AND cap framerate
        present_mode: wgpu::PresentMode::Fifo,
    };

    let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

    // Initialize render passes

    let mesh_pass = MeshPass::new(&mut device, &swap_chain_desc, world, resources)?;

    Ok((
        Graphics {
            device: Rc::new(device),
            queue: Rc::new(queue),
            window: Rc::new(window),
            mesh_pass,
            swap_chain,
            sc_desc: swap_chain_desc,
            surface,
        },
        event_loop,
    ))
}
