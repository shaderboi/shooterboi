use crate::nalgebra;
use crate::util::clamp;
use rapier3d::na::Vector3;

pub struct Camera {
    pub(crate) position: Vector3<f32>,
    yaw: f32,
    pitch: f32,
    fov: f32,
    sensitivity: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            yaw: 270.0,
            pitch: 0.0,
            fov: 60.0,
            sensitivity: 0.5,
        }
    }

    pub fn get_direction_without_pitch(&self) -> nalgebra::Vector3<f32> {
        nalgebra::Vector3::new(
            self.yaw.to_radians().cos(),
            0.0,
            self.yaw.to_radians().sin(),
        )
        .normalize()
    }

    pub fn move_direction(&mut self, offset: nalgebra::Vector2<f32>) {
        let offset_with_sensitivity = offset * self.sensitivity;
        self.yaw -= offset_with_sensitivity.x;
        self.pitch += offset_with_sensitivity.y;

        self.pitch = clamp(self.pitch, -89.0, 89.0);
    }

    pub fn get_direction(&self) -> nalgebra::Vector3<f32> {
        nalgebra::Vector3::new(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        )
        .normalize()
    }

    pub fn get_direction_right(&self) -> nalgebra::Vector3<f32> {
        self.get_direction_without_pitch()
            .cross(&nalgebra::Vector3::new(0.0, 1.0, 0.0))
            .normalize()
    }
}
