use legion::{Resources, World};

use crate::ui::debug::CameraDebugWindow;

#[derive(Default, Debug)]
pub struct Console {
    pub open: bool,
    log: String,
    command: String,
}

impl Console {
    pub fn update(&mut self, world: &mut World, resources: &mut Resources) -> eyre::Result<()> {
        if self.open {
            let mut cmd_complete = false;
            {
                let ctx = resources.get::<egui::CtxRef>().unwrap();
                egui::Window::new("Console").show(&ctx, |ui| -> eyre::Result<()> {
                    ui.label("Dev console");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.log)
                            .code_editor()
                            .interactive(false)
                            .cursor_at_end(true),
                    );
                    let resp = ui.text_edit_singleline(&mut self.command);
                    if resp.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                        log::warn!("Text: {:?}", self.command);
                        cmd_complete = true;
                    };
                    Ok(())
                });
            }
            if cmd_complete {
                self.handle_command(world, resources)?;
            }
        }
        Ok(())
    }
    fn handle_command(&mut self, world: &mut World, resources: &mut Resources) -> eyre::Result<()> {
        self.log.push_str(&format!("CMD: {}\n", &self.command));

        match self.command.trim() {
            "clear" => self.log.clear(),
            "josh off" => {
                use legion::query::IntoQuery;
                let mut entt = None;
                {
                    let mut query = <(legion::Entity, &engine::spacetime::Position)>::query();
                    for (ent, pos) in query.iter(world) {
                        if pos.past().translation.vector.x == 2.1 {
                            entt = Some(*ent);
                        }
                    }
                }
                if let Some(ent) = entt {
                    world.remove(ent);
                    self.log.push_str("Josh successfully obliterated!\n");
                } else {
                    log::error!("NO josh found!");
                }
            }
            "show debug" => {
                let cam_debug = resources.insert(CameraDebugWindow);
            }
            _ => (),
        }
        self.command.clear();
        Ok(())
    }
}
