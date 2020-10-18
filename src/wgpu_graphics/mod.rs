use winit::{
    event_loop::EventLoop,
    window::Window,
};
use anyhow::Result;
use legion::{World, Resources};

pub mod mesh;
mod pass;
pub use pass::Pass;
mod camera;
pub use camera::Camera;

pub async fn setup(world: &mut World, resources: &mut Resources) -> Result<(
    wgpu::Device,
    wgpu::SwapChain,
    wgpu::SwapChainDescriptor,
    wgpu::Surface,
    wgpu::Queue,
    winit::window::Window,
    winit::event_loop::EventLoop<()>,
)> {
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

    let swap_chain_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        // Wait for vsync, but do not cap framerate
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);
    
    Ok((
        device,
        swap_chain,
        swap_chain_desc,
        surface,
        queue,
        window,
        event_loop,
    ))
}