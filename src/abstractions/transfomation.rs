use lunar_lib::math::{mat4x4::Mat4x4, vec3::Vec3};

#[derive(Debug)]
pub struct Transformation {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Default for Transformation {
    fn default() -> Self {
        Self {
            position: Vec3::default(),
            rotation: Vec3::default(),
            scale: Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        }
    }
}

impl Transformation {
    #[must_use]
    pub fn matrix(&self) -> Mat4x4 {
        Mat4x4::transform_matrix_euler(&self.position, &self.scale, &self.rotation)
    }
}
