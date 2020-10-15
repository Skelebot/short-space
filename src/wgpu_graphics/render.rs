
fn setup(world: &mut World, ) {
    // Set up the camera
    let camera = Camera::new(viewport.get_aspect(), 3.14/2.0, 0.01, 1000.0); 
    resources.insert(camera);
}