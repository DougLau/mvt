// geom.rs
//
// Copyright (c) 2017-2018  Douglas P Lau
//
//! Basic 2D geometry -- Vec2 and Transform.
//!
use std::ops;

/// 2-dimensional vector / point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

/// An affine transform can translate, scale, rotate and skew 2D points.
///
/// A series of transforms can be combined into a single Transform struct.
///
/// # Example
/// ```
/// use mvt::Transform;
/// const PI: f64 = std::f64::consts::PI;
/// let t = Transform::new_translate(-50.0, -50.0)
///                   .rotate(PI)
///                   .translate(50.0, 50.0)
///                   .scale(2.0, 2.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    e: [f64; 6],
}

impl ops::Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl ops::Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl ops::Mul<f64> for Vec2 {
    type Output = Self;

    fn mul(self, s: f64) -> Self {
        Vec2::new(self.x * s, self.y * s)
    }
}

impl ops::Mul for Vec2 {
    type Output = f64;

    /// Calculate the cross product of two Vec2
    fn mul(self, other: Self) -> f64 {
        self.x * other.y - self.y * other.x
    }
}

impl ops::Div<f64> for Vec2 {
    type Output = Self;

    fn div(self, s: f64) -> Self {
        Vec2::new(self.x / s, self.y / s)
    }
}

impl ops::Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self {
        Vec2::new(-self.x, -self.y)
    }
}

impl Vec2 {
    /// Create a new Vec2
    pub fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }
    /// Create a zero Vec2
    pub fn zero() -> Self {
        Vec2::new(0.0, 0.0)
    }
    /// Get the magnitude of a Vec2
    pub fn mag(self) -> f64 {
        self.x.hypot(self.y)
    }
    /// Create a copy normalized to unit length
    pub fn normalize(self) -> Self {
        let m = self.mag();
        if m > 0.0 {
            self / m
        } else {
            Vec2::zero()
        }
    }
    /// Calculate the distance squared between two Vec2
    pub fn dist_sq(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
    /// Calculate the distance between two Vec2
    pub fn dist(self, other: Self) -> f64 {
        self.dist_sq(other).sqrt()
    }
}

impl ops::MulAssign for Transform {
    fn mul_assign(&mut self, other: Self) {
        self.e = self.mul_e(&other);
    }
}

impl ops::Mul for Transform {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let e = self.mul_e(&other);
        Transform { e }
    }
}

impl ops::Mul<Vec2> for Transform {
    type Output = Vec2;

    fn mul(self, s: Vec2) -> Vec2 {
        let x = self.e[0] * s.x + self.e[1] * s.y + self.e[2];
        let y = self.e[3] * s.x + self.e[4] * s.y + self.e[5];
        Vec2::new(x, y)
    }
}

impl Transform {
    /// Create a new identity transform.
    pub fn new() -> Self {
        Transform {
            e: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        }
    }
    /// Multiple two affine transforms.
    fn mul_e(&self, other: &Self) -> [f64; 6] {
        let mut e = [0.0; 6];
        e[0] = self.e[0] * other.e[0] + self.e[3] * other.e[1];
        e[1] = self.e[1] * other.e[0] + self.e[4] * other.e[1];
        e[2] = self.e[2] * other.e[0] + self.e[5] * other.e[1] + other.e[2];
        e[3] = self.e[0] * other.e[3] + self.e[3] * other.e[4];
        e[4] = self.e[1] * other.e[3] + self.e[4] * other.e[4];
        e[5] = self.e[2] * other.e[3] + self.e[5] * other.e[4] + other.e[5];
        e
    }
    /// Create a new translation transform.
    ///
    /// * `tx` Amount to translate X.
    /// * `ty` Amount to translate Y.
    pub fn new_translate(tx: f64, ty: f64) -> Self {
        Transform {
            e: [1.0, 0.0, tx, 0.0, 1.0, ty],
        }
    }
    /// Create a new scale transform.
    ///
    /// * `sx` Scale factor for X dimension.
    /// * `sy` Scale factor for Y dimension.
    pub fn new_scale(sx: f64, sy: f64) -> Self {
        Transform {
            e: [sx, 0.0, 0.0, 0.0, sy, 0.0],
        }
    }
    /// Create a new rotation transform.
    ///
    /// * `th` Angle to rotate coordinates (radians).
    pub fn new_rotate(th: f64) -> Self {
        let sn = th.sin();
        let cs = th.cos();
        Transform {
            e: [cs, -sn, 0.0, sn, cs, 0.0],
        }
    }
    /// Create a new skew transform.
    ///
    /// * `ax` Angle to skew X-axis (radians).
    /// * `ay` Angle to skew Y-axis (radians).
    pub fn new_skew(ax: f64, ay: f64) -> Self {
        let tnx = ax.tan();
        let tny = ay.tan();
        Transform {
            e: [1.0, tnx, 0.0, tny, 1.0, 0.0],
        }
    }
    /// Apply translation to a transform.
    ///
    /// * `tx` Amount to translate X.
    /// * `ty` Amount to translate Y.
    pub fn translate(mut self, tx: f64, ty: f64) -> Self {
        self *= Transform::new_translate(tx, ty);
        self
    }
    /// Apply scaling to a transform.
    ///
    /// * `sx` Scale factor for X dimension.
    /// * `sy` Scale factor for Y dimension.
    pub fn scale(mut self, sx: f64, sy: f64) -> Self {
        self *= Transform::new_scale(sx, sy);
        self
    }
    /// Apply rotation to a transform.
    ///
    /// * `th` Angle to rotate coordinates (radians).
    pub fn rotate(mut self, th: f64) -> Self {
        self *= Transform::new_rotate(th);
        self
    }
    /// Apply skew to a transform.
    ///
    /// * `ax` Angle to skew X-axis (radians).
    /// * `ay` Angle to skew Y-axis (radians).
    pub fn skew(mut self, ax: f64, ay: f64) -> Self {
        self *= Transform::new_skew(ax, ay);
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::f64;
    #[test]
    fn test_vec2() {
        let a = Vec2::new(2.0, 1.0);
        let b = Vec2::new(3.0, 4.0);
        assert_eq!(a + b, Vec2::new(5.0, 5.0));
        assert_eq!(b - a, Vec2::new(1.0, 3.0));
        assert_eq!(a * 2.0, Vec2::new(4.0, 2.0));
        assert_eq!(a / 2.0, Vec2::new(1.0, 0.5));
        assert_eq!(-a, Vec2::new(-2.0, -1.0));
        assert_eq!(b.mag(), 5.0);
        assert_eq!(a.dist_sq(b), 10.0);
        assert_eq!(b.dist(Vec2::new(0.0, 0.0)), 5.0);
    }
    #[test]
    fn test_identity() {
        assert_eq!(Transform::new().e, [1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
        assert_eq!(
            (Transform::new() * Transform::new()).e,
            [1.0, 0.0, 0.0, 0.0, 1.0, 0.0]
        );
        assert_eq!(Transform::new() * Vec2::new(1.0, 2.0), Vec2::new(1.0, 2.0));
    }
    #[test]
    fn test_translate() {
        assert_eq!(
            Transform::new_translate(1.5, -1.5).e,
            [1.0, 0.0, 1.5, 0.0, 1.0, -1.5]
        );
        assert_eq!(
            Transform::new().translate(2.5, -3.5).e,
            [1.0, 0.0, 2.5, 0.0, 1.0, -3.5]
        );
        assert_eq!(
            Transform::new().translate(5.0, 7.0) * Vec2::new(1.0, -2.0),
            Vec2::new(6.0, 5.0)
        );
    }
    #[test]
    fn test_scale() {
        assert_eq!(
            Transform::new_scale(2.0, 4.0).e,
            [2.0, 0.0, 0.0, 0.0, 4.0, 0.0]
        );
        assert_eq!(
            Transform::new().scale(3.0, 5.0).e,
            [3.0, 0.0, 0.0, 0.0, 5.0, 0.0]
        );
        assert_eq!(
            Transform::new().scale(2.0, 3.0) * Vec2::new(1.5, -2.0),
            Vec2::new(3.0, -6.0)
        );
    }
    #[test]
    fn test_skew() {
        const PI: f64 = f64::consts::PI;
        assert_eq!(
            Transform::new().skew(0.0, PI / 4.0) * Vec2::new(15.0, 7.0),
            Vec2::new(15.0, 22.0)
        );
    }
    #[test]
    fn test_transform() {
        assert_eq!(
            (Transform::new_translate(1.0, 2.0) * Transform::new_scale(2.0, 2.0)).e,
            [2.0, 0.0, 2.0, 0.0, 2.0, 4.0]
        );
        assert_eq!(
            Transform::new_translate(3.0, 5.0)
                * Transform::new_scale(7.0, 11.0)
                * Transform::new_rotate(f64::consts::PI / 2.0)
                * Transform::new_skew(1.0, -2.0),
            Transform::new()
                .translate(3.0, 5.0)
                .scale(7.0, 11.0)
                .rotate(f64::consts::PI / 2.0)
                .skew(1.0, -2.0)
        );
    }
}
