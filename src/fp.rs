use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use fixed::traits::ToFixed;

pub type FP128 = fixed::types::I96F32;

/// fixed point 128-bit vector
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Vec3F {
    pub x: FP128,
    pub y: FP128,
    pub z: FP128,
}

#[allow(unused)]
impl Vec3F {
    pub const ZERO: Self = Self::splat(fixed!(0.0: I96F32));
    pub const ONE: Self = Self::splat(fixed!(1.0: I96F32));
    pub const N_ONE: Self = Self::splat(fixed!(-1.0: I96F32));
    pub const X: Self = Self::new(fixed!(1.0: I96F32), fixed!(0.0: I96F32), fixed!(0.0: I96F32));
    pub const N_X: Self = Self::new(fixed!(-1.0: I96F32), fixed!(0.0: I96F32), fixed!(0.0: I96F32));
    pub const Y: Self = Self::new(fixed!(0.0: I96F32), fixed!(1.0: I96F32), fixed!(0.0: I96F32));
    pub const N_Y: Self = Self::new(fixed!(0.0: I96F32), fixed!(-1.0: I96F32), fixed!(0.0: I96F32));
    pub const Z: Self = Self::new(fixed!(0.0: I96F32), fixed!(0.0: I96F32), fixed!(1.0: I96F32));
    pub const N_Z: Self = Self::new(fixed!(0.0: I96F32), fixed!(0.0: I96F32), fixed!(-1.0: I96F32));

    pub const fn new(x: FP128, y: FP128, z: FP128) -> Self {
        Self { x, y, z }
    }

    pub const fn splat(f: FP128) -> Self {
        Self { x: f, y: f, z: f }
    }

    pub const fn from_slice(arr: &[FP128]) -> Self {
        Self {
            x: arr[0],
            y: arr[1],
            z: arr[2],
        }
    }

    pub const fn to_array(&self) -> [FP128; 3] {
        [self.x, self.y, self.z]
    }

    pub fn from_f32s(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: x.to_fixed(),
            y: y.to_fixed(),
            z: z.to_fixed(),
        }
    }

    pub fn to_f32s(self) -> (f32, f32, f32) {
        (self.x.to_num(), self.y.to_num(), self.z.to_num())
    }

    pub fn from_f64s(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: x.to_fixed(),
            y: y.to_fixed(),
            z: z.to_fixed(),
        }
    }

    pub fn to_f64s(self) -> (f64, f64, f64) {
        (self.x.to_num(), self.y.to_num(), self.z.to_num())
    }

    pub fn from_vec3(value: glam::Vec3) -> Self {
        Self {
            x: value.x.to_fixed(),
            y: value.y.to_fixed(),
            z: value.z.to_fixed(),
        }
    }
    
    pub fn to_vec3(self) -> glam::Vec3 {
        glam::Vec3 {
            x: self.x.to_num(),
            y: self.y.to_num(),
            z: self.z.to_num(),
        }
    }
    
    pub fn from_dvec3(value: glam::DVec3) -> Self {
        Self {
            x: value.x.to_fixed(),
            y: value.y.to_fixed(),
            z: value.z.to_fixed(),
        }
    }
    
    pub fn to_dvec3(self) -> glam::DVec3 {
        glam::DVec3 {
            x: self.x.to_num(),
            y: self.y.to_num(),
            z: self.z.to_num(),
        }
    }

    pub fn dot(&self, other: Vec3F) -> FP128 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: Vec3F) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn length(&self) -> FP128 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> FP128 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn max(&self) -> FP128 {
        self.x.max(self.y.max(self.z))
    }

    pub fn abs(&self) -> Vec3F {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }

    // pub fn compress(&self, origin: Vec3F) -> glam::Vec3 {
    //     let diff = *self - origin;
    //     let len = (FP128::ONE + diff.max()).sqrt() - FP128::ONE;
    //     let diff = diff / diff.max() * len;
        
    //     glam::Vec3 {
    //         x: diff.x.to_num(),
    //         y: diff.y.to_num(),
    //         z: diff.z.to_num(),
    //     }
    // }
}

impl From<(f32, f32, f32)> for Vec3F {
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self::from_f32s(x, y, z)
    }
}

impl Into<(f32, f32, f32)> for Vec3F {
    fn into(self) -> (f32, f32, f32) {
        self.to_f32s()
    }
}

impl From<(f64, f64, f64)> for Vec3F {
    fn from((x, y, z): (f64, f64, f64)) -> Self {
        Self::from_f64s(x, y, z)
    }
}

impl Into<(f64, f64, f64)> for Vec3F {
    fn into(self) -> (f64, f64, f64) {
        self.to_f64s()
    }
}

impl From<[f32; 3]> for Vec3F {
    fn from([x, y, z]: [f32; 3]) -> Self {
        Self::from_f32s(x, y, z)
    }
}

impl Into<[f32; 3]> for Vec3F {
    fn into(self) -> [f32; 3] {
        self.to_f32s().into()
    }
}

impl From<[f64; 3]> for Vec3F {
    fn from([x, y, z]: [f64; 3]) -> Self {
        Self::from_f64s(x, y, z)
    }
}

impl Into<[f64; 3]> for Vec3F {
    fn into(self) -> [f64; 3] {
        self.to_f64s().into()
    }
}

impl From<glam::Vec3> for Vec3F {
    fn from(value: glam::Vec3) -> Self {
        Self::from_vec3(value)
    }
}

impl Into<glam::Vec3> for Vec3F {
    fn into(self) -> glam::Vec3 {
        self.to_vec3()
    }
}

impl From<glam::DVec3> for Vec3F {
    fn from(value: glam::DVec3) -> Self {
        Self::from_dvec3(value)
    }
}

impl Into<glam::DVec3> for Vec3F {
    fn into(self) -> glam::DVec3 {
        self.to_dvec3()
    }
}

impl From<&[FP128]> for Vec3F {
    fn from(value: &[FP128]) -> Self {
        Self::from_slice(value)
    }
}

impl Into<[FP128; 3]> for Vec3F {
    fn into(self) -> [FP128; 3] {
        self.to_array()
    }
}

impl Add for Vec3F {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vec3F {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul for Vec3F {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Mul<FP128> for Vec3F {
    type Output = Self;

    fn mul(self, rhs: FP128) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<f32> for Vec3F {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs.to_fixed::<FP128>(),
            y: self.y * rhs.to_fixed::<FP128>(),
            z: self.z * rhs.to_fixed::<FP128>(),
        }
    }
}

impl Mul<f64> for Vec3F {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs.to_fixed::<FP128>(),
            y: self.y * rhs.to_fixed::<FP128>(),
            z: self.z * rhs.to_fixed::<FP128>(),
        }
    }
}

impl Div<FP128> for Vec3F {
    type Output = Self;

    fn div(self, rhs: FP128) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Div<f32> for Vec3F {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs.to_fixed::<FP128>(),
            y: self.y / rhs.to_fixed::<FP128>(),
            z: self.z / rhs.to_fixed::<FP128>(),
        }
    }
}

impl Div<f64> for Vec3F {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x / rhs.to_fixed::<FP128>(),
            y: self.y / rhs.to_fixed::<FP128>(),
            z: self.z / rhs.to_fixed::<FP128>(),
        }
    }
}

impl AddAssign for Vec3F {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl SubAssign for Vec3F {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl MulAssign for Vec3F {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl MulAssign<FP128> for Vec3F {
    fn mul_assign(&mut self, rhs: FP128) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl MulAssign<f32> for Vec3F {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs.to_fixed::<FP128>();
        self.y *= rhs.to_fixed::<FP128>();
        self.z *= rhs.to_fixed::<FP128>();
    }
}

impl MulAssign<f64> for Vec3F {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs.to_fixed::<FP128>();
        self.y *= rhs.to_fixed::<FP128>();
        self.z *= rhs.to_fixed::<FP128>();
    }
}

impl DivAssign<FP128> for Vec3F {
    fn div_assign(&mut self, rhs: FP128) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl DivAssign<f32> for Vec3F {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs.to_fixed::<FP128>();
        self.y /= rhs.to_fixed::<FP128>();
        self.z /= rhs.to_fixed::<FP128>();
    }
}

impl DivAssign<f64> for Vec3F {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs.to_fixed::<FP128>();
        self.y /= rhs.to_fixed::<FP128>();
        self.z /= rhs.to_fixed::<FP128>();
    }
}

pub trait MulVec3F {
    fn mul_vec3f(&self, other: Vec3F) -> Vec3F;
}

impl MulVec3F for glam::Quat {
    fn mul_vec3f(&self, other: Vec3F) -> Vec3F {
        let xyz = Vec3F::new(self.x.to_fixed(), self.y.to_fixed(), self.z.to_fixed());
        other + xyz.cross(xyz.cross(other) + other * self.w) * 2.0
    }
}

impl Neg for Vec3F {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Default for Vec3F {
    fn default() -> Self {
        Self::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init() {
        let f1 = fixed!(1.0: I96F32);
        let f0 = fixed!(0.0: I96F32);

        // default
        assert_eq!(Vec3F::ZERO, Vec3F::default());

        // splat
        assert_eq!(Vec3F::ZERO, Vec3F::splat(f0));
        assert_eq!(Vec3F::ONE, Vec3F::splat(f1));
        assert_eq!(Vec3F::N_ONE, Vec3F::splat(-f1));
        assert_eq!(Vec3F::X, Vec3F::new(f1, f0, f0));
        assert_eq!(Vec3F::N_X, Vec3F::new(-f1, f0, f0));

        // new
        assert_eq!(Vec3F::Y, Vec3F::new(f0, f1, f0));
        assert_eq!(Vec3F::N_Y, Vec3F::new(f0, -f1, f0));
        assert_eq!(Vec3F::Z, Vec3F::new(f0, f0, f1));
        assert_eq!(Vec3F::N_Z, Vec3F::new(f0, f0, -f1));

        let x123 = Vec3F::new(f1, 2*f1, 3*f1);

        // from/into
        assert_eq!(x123, Vec3F::from_slice(&[f1, fixed!(2.0: I96F32), fixed!(3.0: I96F32)]));
        assert_eq!([f1, 2*f1, 3*f1], x123.to_array());
        assert_eq!(x123, glam::Vec3::new(1.0, 2.0, 3.0).into());
        assert_eq!(Into::<glam::Vec3>::into(x123), glam::Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn mul() {
        let f1 = fixed!(1.0: I96F32);
        let f0 = fixed!(0.0: I96F32);
        let x123 = Vec3F::new(f1, 2*f1, 3*f1);

        // dot
        assert_eq!(f1, x123.dot(Vec3F::X));
        assert_eq!(fixed!(2.0: I96F32), x123.dot(Vec3F::Y));
        assert_eq!(fixed!(3.0: I96F32), x123.dot(Vec3F::Z));
        assert_eq!(-f1, x123.dot(Vec3F::N_X));
        assert_eq!(fixed!(-2.0: I96F32), x123.dot(Vec3F::N_Y));
        assert_eq!(fixed!(-3.0: I96F32), x123.dot(Vec3F::N_Z));

        // cross
        assert_eq!(Vec3F::X, Vec3F::Y.cross(Vec3F::Z));
        assert_eq!(Vec3F::Y, Vec3F::Z.cross(Vec3F::X));
        assert_eq!(Vec3F::Z, Vec3F::X.cross(Vec3F::Y));
        assert_eq!(Vec3F::new(f1, f1, f1), Vec3F::new(f0, f1, -f1).cross(Vec3F::new(-f1, f1, f0)));
    }

    #[test]
    fn arithmetic() {
        let f1 = FP128::ONE;
        let x123 = Vec3F::new(f1, 2*f1, 3*f1);
        let x246 = Vec3F::new(2*f1, 4*f1, 6*f1);
        let x149 = Vec3F::new(f1, 4*f1, 9*f1);
        
        // add
        assert_eq!(x246, x123 + x123);
        // sub
        assert_eq!(Vec3F::ZERO, x123 - x123);
        // mul
        assert_eq!(x149, x123 * x123);

        // scalar mul
        assert_eq!(x246, x123 * fixed!(2.0: I96F32));
        assert_eq!(x246, x123 * 2.0f32);
        assert_eq!(x246, x123 * 2.0f64);
        
        // scalar div
        assert_eq!(x123, x246 / fixed!(2.0: I96F32));
        assert_eq!(x123, x246 / 2.0f32);
        assert_eq!(x123, x246 / 2.0f64);
        
        // add assign
        let mut v = x123;
        v += x123;
        assert_eq!(x246, v);

        // sub assign
        let mut v = x123;
        v -= x123;
        assert_eq!(Vec3F::ZERO, v);

        // mul assign
        let mut v = x123;
        v *= x123;
        assert_eq!(x149, v);

        // scalar mul assign
        let mut v = x123;
        v *= fixed!(2.0: I96F32);
        assert_eq!(x246, v);

        let mut v = x123;
        v *= 2.0f32;
        assert_eq!(x246, v);

        let mut v = x123;
        v *= 2.0f64;
        assert_eq!(x246, v);

        // scalar div assign
        let mut v = x246;
        v /= fixed!(2.0: I96F32);
        assert_eq!(x123, v);

        let mut v = x246;
        v /= 2.0f32;
        assert_eq!(x123, v);

        let mut v = x246;
        v /= 2.0f64;
        assert_eq!(x123, v);

        // quat mul
        assert!((glam::Quat::from_axis_angle(glam::Vec3::Y, std::f32::consts::FRAC_PI_2).mul_vec3f(Vec3F::X) - Vec3F::N_Z).length() < 0.00001);

        // neg
        let n123 = Vec3F::new(-f1, -2*f1, -3*f1);
        assert_eq!(n123, -x123);
    }
}
