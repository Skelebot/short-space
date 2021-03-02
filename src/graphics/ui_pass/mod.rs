use eyre::Result;
use imgui::{im_str, Context};
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

        // TODO: Move everything that happens here into a config file.
        // Dear-ImGui doesn't really provide anything that would help
        // with loading configs, so we have to set every option by hand.
        // scope just for folding
        {
            //{
            //    let assets = resources.get::<AssetLoader>().unwrap();
            //    // ???
            //    // apparently this does nothing useful
            //    imgui.load_ini_settings(&assets.load_str("settings/imgui.ini")?)
            //}
            imgui.fonts().add_font(&[imgui::FontSource::TtfData {
                data: include_bytes!("../../../assets/fonts/PTSansNarrow-Regular.ttf"),
                size_pixels: 18.0,
                config: Some(imgui::FontConfig {
                    oversample_h: 2,
                    oversample_v: 2,
                    ..Default::default()
                }),
            }]);
            let round = 3.0;
            let border = 0.0;
            let padding = 8.0;
            imgui.style_mut().alpha = 1.0;
            imgui.style_mut().window_padding = [padding, padding];
            imgui.style_mut().window_rounding = round;
            imgui.style_mut().window_border_size = border;
            imgui.style_mut().window_min_size = [50.0, 50.0];
            imgui.style_mut().window_title_align = [0.5, 0.5];
            imgui.style_mut().window_menu_button_position = imgui::Direction::Right;
            imgui.style_mut().child_rounding = round;
            imgui.style_mut().frame_padding = [2.0, 2.0];
            imgui.style_mut().child_border_size = border;
            imgui.style_mut().popup_rounding = round;
            imgui.style_mut().popup_border_size = border;
            imgui.style_mut().frame_rounding = round;
            imgui.style_mut().frame_border_size = border;
            imgui.style_mut().scrollbar_size = 5.0;
            imgui.style_mut().scrollbar_rounding = round;
            imgui.style_mut().tab_rounding = round;
            imgui.style_mut().tab_border_size = border;
            imgui.style_mut().anti_aliased_lines = false;
            imgui.style_mut().anti_aliased_lines_use_tex = false;
            imgui.style_mut().anti_aliased_fill = false;
            // TODO: After loading from a file is done, make a nice colorscheme instead of this
            let accent = super::color::Rgba::new(0.85, 0.62, 0.0, 1.0);
            let gray = super::color::Rgba::new(0.15, 0.15, 0.15, 1.0);
            imgui.style_mut()[imgui::StyleColor::TitleBgActive] = gray.into();
            imgui.style_mut()[imgui::StyleColor::Button] = gray.into();
            imgui.style_mut()[imgui::StyleColor::FrameBg] = accent.into();
            imgui.style_mut()[imgui::StyleColor::FrameBgActive] = accent.into();
            imgui.style_mut()[imgui::StyleColor::TitleBgActive] = accent.into();

            imgui.style_mut()[imgui::StyleColor::WindowBg] =
                super::color::Rgba::new(0.08, 0.08, 0.08, 1.0).into();
        }
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
        resources: &legion::Resources,
    ) {
        self.platform
            .prepare_frame(self.ctx.io_mut(), &graphics.window)
            .unwrap();
        let ui = self.ctx.frame();
        // IMGUI
        self.platform.prepare_render(&ui, &graphics.window);
        {
            if let Some(mut start) = resources.get_mut::<crate::ui::StartWindow>() {
                if start.opened {
                    imgui::Window::new(im_str!(" Main menu"))
                        .size([200.0, 120.0], imgui::Condition::Once)
                        .collapsible(false)
                        .resizable(false)
                        .build(&ui, || {
                            start.play_pressed = ui.button(im_str!("Start"), [150.0, 30.0]);
                            start.exit_pressed = ui.button(im_str!("Exit"), [150.0, 30.0]);
                        });
                }
            }
            //ui.show_demo_window(&mut true);
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
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
