use egui::Ui;
use engine::{graphics::MainCamera, spacetime::PhysicsTimer};

pub struct CameraDebugWindow;

impl CameraDebugWindow {
    pub fn update(&mut self, world: &mut legion::World, resources: &legion::Resources) {
        let ctx = resources.get::<egui::CtxRef>().unwrap();
        let cam = resources.get::<MainCamera>();
        let lerp = resources
            .get::<PhysicsTimer>()
            .map(|t| t.lerp())
            .unwrap_or(1.0);
        egui::Window::new("Camera debug").show(&ctx, |ui| -> eyre::Result<()> {
            if let Some(camera) = cam {
                let pos = camera.position.current(lerp as f32);
                ui.label("Coords: ");
                ui.label(&format!("x: {}", pos.translation.vector.x));
                ui.label(&format!("y: {}", pos.translation.vector.y));
                ui.label(&format!("z: {}", pos.translation.vector.z));
                ui.label(&format!("rotation: {:?}", pos.rotation.as_vector()));
            } else {
                ui.label("No camera present.");
            }
            Ok(())
        });
    }
}
