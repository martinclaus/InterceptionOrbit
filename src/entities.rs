//! Core bussiness objects of Interception Orbit

use std::ops::{Add, Div, Mul, Sub};

pub const PI: f32 = std::f32::consts::PI;
pub const TWO_PI: f32 = 2_f32 * PI;

/// Cartesian 2D Vector type
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    /// Create a new 2D vector
    pub fn new(x: impl Into<f32>, y: impl Into<f32>) -> Self {
        Vec2 {
            x: x.into(),
            y: y.into(),
        }
    }

    /// Square of the length of a [`Vec2`]
    pub fn len2(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Length of a [`Vec2`]
    pub fn len(self) -> f32 {
        self.len2().sqrt()
    }

    /// Return new [`Vec2`] with same direction but unit length
    pub fn norm(self) -> Vec2 {
        let len = self.len();
        Vec2 {
            x: self.x / len,
            y: self.y / len,
        }
    }
}

impl From<PolarVec2> for Vec2 {
    fn from(value: PolarVec2) -> Self {
        Vec2::new(value.angle.cos() * value.len, value.angle.sin() * value.len)
    }
}

impl Add<Vec2> for Vec2 {
    type Output = Vec2;

    /// Vector addition
    fn add(self, rhs: Vec2) -> Self::Output {
        Self::Output::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub<Vec2> for Vec2 {
    type Output = Vec2;

    /// Vector difference
    fn sub(self, rhs: Vec2) -> Self::Output {
        Self::Output::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    /// Scalar multiplication
    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    /// Scalar multiplication
    fn mul(self, rhs: Vec2) -> Self::Output {
        rhs * self
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;

    /// Scalar division
    fn div(self, rhs: f32) -> Self::Output {
        if rhs == 0.0 {
            Self::Output::new(0.0, 0.0)
        } else {
            self * rhs.recip()
        }
    }
}

impl Mul<Vec2> for Vec2 {
    type Output = f32;

    fn mul(self, rhs: Vec2) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y
    }
}

/// Polar 2D Vector type
///
/// This vector has two components, its length and the angle. The angle is within [0, 2*PI].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PolarVec2 {
    len: f32,
    angle: f32,
}

impl PolarVec2 {
    /// Create a new instance
    pub fn new(len: impl Into<f32>, angle: impl Into<f32>) -> Self {
        PolarVec2 {
            len: len.into(),
            angle: trim_angle(angle),
        }
    }

    /// Rotate vector by angle in radians
    pub fn rotate(self, angle: impl Into<f32>) -> Self {
        Self::new(self.len, self.angle + angle.into())
    }
}

impl From<Vec2> for PolarVec2 {
    fn from(value: Vec2) -> Self {
        PolarVec2::new(value.len(), value.y.atan2(value.x))
    }
}

/// Trim angle in radians to [0, 2*PI]
pub fn trim_angle(angle: impl Into<f32>) -> f32 {
    let mut result = angle.into() % TWO_PI;
    while result < 0_f32 {
        result += TWO_PI
    }
    result
}

#[cfg(test)]
mod test {

    use std::f32::EPSILON;

    use super::{trim_angle, PolarVec2, Vec2, PI, TWO_PI};

    #[test]
    fn vec2_can_be_created_with_other_input() {
        assert_eq!(Vec2::new(1_i16, 0.0), Vec2::new(1.0, 0.0))
    }

    #[test]
    fn vec2_len_is_correct() {
        assert_eq!(Vec2::new(2.0, 0.0).len(), 2.0);
        assert_eq!(Vec2::new(0.0, 2.0).len(), 2.0);
        assert_eq!(Vec2::new(0.0, -2.0).len(), 2.0);
        assert_eq!(Vec2::new(5.0, -2.0).len(), (29.0_f32).sqrt());
    }

    #[test]
    fn vec2_normalizes_correct() {
        assert_eq!(Vec2::new(5.0, -2.0).norm().len(), 1.0);
    }

    #[test]
    fn angle_trimmed_correctly() {
        assert_eq!(trim_angle(0.5), 0.5);
        assert_eq!(trim_angle(TWO_PI + 0.5), 0.5);
        assert_eq!(trim_angle(-2.0 * TWO_PI - 0.5), TWO_PI - 0.5);
    }

    #[test]
    fn polar_vec2_trims_angle() {
        assert_eq!(PolarVec2::new(1.0, TWO_PI + 0.5), PolarVec2::new(1.0, 0.5))
    }

    #[test]
    fn polar_vec2_rotation_trims_angle() {
        assert_eq!(
            PolarVec2::new(1.0, PI).rotate(1.5 * PI),
            PolarVec2::new(1.0, trim_angle(PI + 1.5 * PI))
        )
    }

    #[test]
    fn vec2_converts_to_polar_vec2() {
        assert_eq!(
            <Vec2 as Into<PolarVec2>>::into(Vec2::new(1.0, 0.0)),
            PolarVec2::new(1.0, 0.0)
        );
        assert_eq!(
            <Vec2 as Into<PolarVec2>>::into(Vec2::new(0.0, -2.0)),
            PolarVec2::new(2.0, 1.5 * PI)
        );
    }

    #[test]
    fn polar_vec2_converts_to_vec2() {
        assert_eq!(
            Vec2::new(1.0, 0.0),
            <PolarVec2 as Into<Vec2>>::into(PolarVec2::new(1.0, 0.0))
        );

        assert!(
            (Vec2::new(0.0, -2.0) - <PolarVec2 as Into<Vec2>>::into(PolarVec2::new(2.0, 1.5 * PI)))
                .len()
                < EPSILON
        );
    }
}
