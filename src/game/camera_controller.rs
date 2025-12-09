use nalgebra::Vector2;
use winit::keyboard::KeyCode;

use super::Camera;
use crate::input::KeyboardBinding;

pub struct CameraController {
    speed: f32,
    drag_sensitivity: f32,
}
impl CameraController {
    pub fn new() -> Self {
        Self {
            speed: 1.0,
            drag_sensitivity: 0.001,
        }
    }

    pub fn move_camera(&self, camera: &mut Camera, input: &KeyboardBinding, dt: f32) {
        let mut forward = 0.0;
        let mut right = 0.0;
        let mut up = 0.0;
        let mut speed = self.speed;
        if input.down(KeyCode::KeyW as usize) {
            forward -= 1.0;
        }
        if input.down(KeyCode::KeyS as usize) {
            forward += 1.0;
        }
        if input.down(KeyCode::KeyD as usize) {
            right += 1.0;
        }
        if input.down(KeyCode::KeyA as usize) {
            right -= 1.0;
        }
        if input.down(KeyCode::Space as usize) {
            up += 1.0;
        }
        if input.down(KeyCode::ControlLeft as usize) {
            up -= 1.0;
        }
        if input.down(KeyCode::ShiftLeft as usize) {
            speed *= 10.0;
        }
        let dir = camera.dir_flat();
        let perp_dir = Vector2::new(dir.y, -dir.x);
        camera.position.x += (perp_dir.x * right + dir.x * forward) * speed * dt;
        camera.position.z += (perp_dir.y * right + dir.y * forward) * speed * dt;
        camera.position.y += up * speed * dt;
    }

    pub fn drag_camera(&self, camera: &mut Camera, delta: Vector2<f32>) {
        let safty = std::f32::consts::FRAC_PI_2 - 0.001;
        camera.pitch = nalgebra::clamp(
            camera.pitch - delta.y * self.drag_sensitivity,
            -safty,
            safty,
        );
        camera.yaw += delta.x * self.drag_sensitivity;
    }
}
