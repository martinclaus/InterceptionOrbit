//! Core bussiness objects of Interception Orbit

use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Sub},
};

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

    /// Zero vector (0.0, 0.0)
    pub fn zero() -> Self {
        Vec2 { x: 0.0, y: 0.0 }
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

    /// Rotate vector by `angle` in radians
    pub fn rotate(self, angle: f32) -> Self {
        let Self { x, y } = self;
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        Vec2::new(cos_angle * x - sin_angle * y, sin_angle * x + cos_angle * y)
    }

    /// getter for x value
    pub fn get_x(&self) -> f32 {
        self.x
    }

    /// getter for x value
    pub fn get_y(&self) -> f32 {
        self.y
    }

    /// Get angle with x-axis in radians
    pub fn angle(&self) -> f32 {
        trim_angle(self.y.atan2(self.x))
    }
}

impl Add<Vec2> for Vec2 {
    type Output = Vec2;

    /// Vector addition
    fn add(self, rhs: Vec2) -> Self::Output {
        Self::Output::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<'a> Sum<&'a Vec2> for Vec2 {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        let mut result = Vec2::new(0.0, 0.0);
        iter.for_each(|item| result += *item);
        result
    }
}

impl Sum<Vec2> for Vec2 {
    fn sum<I: Iterator<Item = Vec2>>(iter: I) -> Self {
        let mut result = Vec2::new(0.0, 0.0);
        iter.for_each(|item| result += item);
        result
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

/// Trim angle in radians to [0, 2*PI]
pub fn trim_angle(angle: impl Into<f32>) -> f32 {
    let mut result = angle.into() % TWO_PI;
    while result < 0_f32 {
        result += TWO_PI
    }
    result
}

/// gravitational acceleration
pub fn gravity(attractant_position: Vec2, attractant_mass: f32, body_position: Vec2) -> Vec2 {
    const GRAVITY_CONSTANT: f32 = 1.0;
    let r = attractant_position - body_position;
    let r2 = r.len2();
    r.norm() * GRAVITY_CONSTANT * attractant_mass / r2
}

#[cfg(test)]
mod test {

    use std::f32::{
        consts::{FRAC_PI_2, FRAC_PI_4},
        EPSILON,
    };

    use super::{gravity, trim_angle, Vec2, PI, TWO_PI};

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
    fn vec2_rotates_correct() {
        assert!((Vec2::new(1.0, 0.0).rotate(PI) - Vec2::new(-1.0, 0.0)).len() < EPSILON);
        assert!((Vec2::new(1.0, 0.0).rotate(PI / 2.0) - Vec2::new(0.0, 1.0)).len() < EPSILON);
        assert!((Vec2::new(1.0, 0.0).rotate(-PI / 2.0) - Vec2::new(0.0, -1.0)).len() < EPSILON);
    }

    #[test]
    fn angle_trimmed_correctly() {
        assert_eq!(trim_angle(0.5), 0.5);
        assert_eq!(trim_angle(TWO_PI + 0.5), 0.5);
        assert_eq!(trim_angle(-2.0 * TWO_PI - 0.5), TWO_PI - 0.5);
    }

    #[test]
    fn angle_of_vec2_correctly_computed() {
        assert!(Vec2::new(1.0, 0.0).angle() == 0.);
        assert!((Vec2::new(0.0, 1.0).angle() - FRAC_PI_2).abs() < EPSILON);
        assert!((Vec2::new(-1.0, 0.0).angle() - PI).abs() < 2.1 * EPSILON);
        assert!((Vec2::new(0.0, -1.0).angle() - 3.0 * FRAC_PI_2).abs() < EPSILON);
        assert!(Vec2::new(1.0, 1.0).angle() - FRAC_PI_4 < EPSILON);
        assert!(Vec2::new(-1.0, -1.0).angle() - 5.0 * FRAC_PI_4 < EPSILON);
    }

    #[test]
    fn vec2_summed_correctly() {
        assert_eq!(
            [
                Vec2::new(1.0, 3.0),
                Vec2::new(2.0, 1.0),
                Vec2::new(5.0, 1.0),
            ]
            .iter()
            .sum::<Vec2>(),
            Vec2::new(8.0, 5.0)
        )
    }

    #[test]
    fn gravity_computed_correctly() {
        let pos1 = Vec2::new(0.0, 0.0);
        let pos2 = Vec2::new(2.0, 0.0);
        let mass = 10.0;
        assert_eq!(gravity(pos1, mass, pos2), Vec2::new(-10.0 / 4.0, 0.0));
    }
}
