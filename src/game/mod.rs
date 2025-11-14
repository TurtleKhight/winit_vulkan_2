pub mod camera;

use camera::Camera;
// use imgui::Ui;

pub struct Game {
    pub camera: Camera,
}
impl Game {}

impl Default for Game {
    fn default() -> Self {
        let camera: Camera = Camera::default();
        Self { camera }
    }
}

// impl Game {
//     pub fn ui(&mut self, ui: &Ui) {
//         ui.group(|| {});
//         ui.slider("Num Instance", 0, 1000, &mut self.instance_len);
//     }
// }
