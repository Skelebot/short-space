use eyre::Result;
use imgui::Context;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use legion::Resources;
use legion::World;
use winit::event::Event;

use crate::state::CustomEvent;

use super::{GraphicsShared, Pass};

pub struct UiPass {
    pub ctx: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
}

impl UiPass {
    pub fn new(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        window: &winit::window::Window,
        queue: &wgpu::Queue,
        _world: &mut World,
        _resources: &mut Resources,
    ) -> Result<Self> {
        // IMGUI
        let mut imgui = Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );
        let renderer = Renderer::new(
            &mut imgui,
            device,
            queue,
            RendererConfig {
                texture_format: sc_desc.format,
                ..Default::default()
            },
        );
        Ok(UiPass {
            ctx: imgui,
            platform,
            renderer,
        })
    }
    pub fn handle_event(&mut self, window: &winit::window::Window, event: &Event<CustomEvent>) {
        self.platform.handle_event(self.ctx.io_mut(), window, event)
    }
}

impl Pass for UiPass {
    fn resize(
        &mut self,
        _graphics: &GraphicsShared,
        _sc_desc: &wgpu::SwapChainDescriptor,
        _world: &mut legion::World,
        _resources: &mut legion::Resources,
    ) -> eyre::Result<()> {
        // Resizing is already handled by self.handle_event()
        Ok(())
    }

    fn render(
        &mut self,
        graphics: &GraphicsShared,
        encoder: &mut wgpu::CommandEncoder,
        target: &mut wgpu::SwapChainTexture,
        _world: &legion::World,
        _resources: &legion::Resources,
    ) {
        self.platform
            .prepare_frame(self.ctx.io_mut(), &graphics.window)
            .unwrap();
        let ui = self.ctx.frame();
        // IMGUI
        self.platform.prepare_render(&ui, &graphics.window);
        {
            //let window = imgui::Window::new(im_str!("Hello World!"));
            //window.size([300.0, 100.0], imgui::Condition::FirstUseEver).build(&ui, || {
            //    ui.text(im_str!("Hello world!"));
            //});
            ui.show_demo_window(&mut true);
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &target.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            self.renderer
                .render(ui.render(), &graphics.queue, &graphics.device, &mut rpass)
                .unwrap();
        }
    }
}
