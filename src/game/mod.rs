pub mod camera;
mod camera_controller;

use camera::Camera;

use crate::game::camera_controller::CameraController;
// use imgui::Ui;

pub struct Game {
    pub camera: Camera,
    pub camera_controller: CameraController,
}
impl Game {}

impl Default for Game {
    fn default() -> Self {
        let camera: Camera = Camera::default();
        let camera_controller = CameraController::new();
        Self {
            camera,
            camera_controller,
        }
    }
}

// impl Game {
//     pub fn ui(&mut self, ui: &Ui) {
//         ui.group(|| {});
//         ui.slider("Num Instance", 0, 1000, &mut self.instance_len);
//     }
// }
