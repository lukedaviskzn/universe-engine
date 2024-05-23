use crate::fp::Vec3F;

pub struct Transform {
    pub translation: Vec3F,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

#[allow(unused)]
impl Transform {
    pub const IDENTITY: Self = Self {
        translation: Vec3F::splat(fixed!(0.0: I96F32)),
        rotation: glam::Quat::IDENTITY,
        scale: glam::Vec3::ONE,
    };

    pub const fn new(translation: Vec3F, rotation: glam::Quat, scale: glam::Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub const fn with_translation(translation: Vec3F) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    pub const fn with_rotation(rotation: glam::Quat) -> Self {
        Self {
            rotation,
            ..Self::IDENTITY
        }
    }

    pub const fn with_scale(scale: glam::Vec3) -> Self {
        Self {
            scale,
            ..Self::IDENTITY
        }
    }

    pub fn matrix(&self, origin: Vec3F) -> glam::Mat4 {
        let translation = self.translation - origin;
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, translation.into())
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn init() {
//         todo!();
//     }
// }
