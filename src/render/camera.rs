use crate::transform::Transform;

pub struct Camera {
    pub transform: Transform,
    pub fovy: f32,
    znear: f32,
}

impl Camera {
    pub fn new(transform: Transform, fovy: f32, znear: f32) -> Self {
        Self {
            transform,
            fovy,
            znear,
        }
    }

    pub fn perspective(&self, aspect: f32) -> glam::Mat4 {
        let perspective = glam::Mat4::perspective_infinite_rh(self.fovy, aspect, self.znear);
        let view = self.transform.matrix(self.transform.translation);

        perspective * view
    }
}
