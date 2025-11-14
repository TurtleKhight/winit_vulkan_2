use nalgebra::{Isometry3, Matrix4, Point3, Vector2, Vector3};

pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub position: Point3<f32>,
    pub znear: f32,
    pub zfar: f32,
    pub fovy: f32,
    pub aspect: f32,
}
impl Camera {
    pub fn dir(&self) -> Vector3<f32> {
        -Vector3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
        .normalize()
    }

    pub fn dir_flat(&self) -> Vector2<f32> {
        -Vector2::new(
            self.yaw.sin() * self.pitch.cos(),
            -self.yaw.cos() * self.pitch.cos(),
        )
        .normalize()
    }

    pub fn mouse_delta(&mut self, mouse_delta: Vector2<f32>) {
        let sensitivity = 0.001;
        let safty = std::f32::consts::FRAC_PI_2 - 0.001;
        self.pitch = nalgebra::clamp(self.pitch - mouse_delta.y * sensitivity, -safty, safty);
        self.yaw += mouse_delta.x * sensitivity;
    }

    pub fn resize(&mut self, new_size: Vector2<u32>) {
        self.aspect = new_size.x as f32 / new_size.y as f32;
    }

    pub fn calc_p_mtx(&self) -> Matrix4<f32> {
        let mut proj = Matrix4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar);
        proj[(1, 1)] *= -1.0;
        return proj;
        // return Matrix4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar);
    }

    pub fn calc_dir_mtx(&self) -> Isometry3<f32> {
        let up = Vector3::new(0.0, 1.0, 0.0);
        Isometry3::face_towards(&Point3::origin(), &self.dir().into(), &up)
    }

    pub fn calc_c_mtx(&self) -> Isometry3<f32> {
        let up = Vector3::new(0.0, 1.0, 0.0);
        Isometry3::face_towards(&self.position, &(self.position + self.dir()), &up)
    }

    pub fn calc_v_mtx(&self) -> Isometry3<f32> {
        self.calc_c_mtx().inverse()
    }
}

impl Default for Camera {
    fn default() -> Self {
        let yaw = 0.0;
        let pitch = 0.0;
        let position = Point3::new(0.0, 0.0, 3.0);
        let znear = 0.01;
        let zfar = 100.0;
        let fovy = (60.0f32).to_radians();
        let aspect = 1.0;
        Self {
            yaw,
            pitch,
            position,
            znear,
            zfar,
            fovy,
            aspect,
        }
    }
}
