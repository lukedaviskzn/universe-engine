use crate::transform::Transform;

pub struct Camera {
    pub transform: Transform,
    pub fovy: f32,
}

impl Camera {
    pub const Z_NEAR: f32 = 1.0;
    
    pub fn new(transform: Transform, fovy: f32) -> Self {
        Self {
            transform,
            fovy,
        }
    }

    pub fn perspective(&self, aspect: f32) -> glam::Mat4 {
        let perspective = glam::Mat4::perspective_infinite_rh(self.fovy, aspect, Self::Z_NEAR);
        let view = self.transform.matrix(self.transform.translation);

        perspective * view
    }
}
