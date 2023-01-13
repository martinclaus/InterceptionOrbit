//! Domain rules for Interception Orbit game

mod entities;
pub mod physics;
pub mod repo;
pub mod user_input;

/// Reexport data gateway interfaces
pub mod repo_interfaces {
    pub use super::user_input::{
        MissileLaunchData, PlayerMovementDataGateway, PlayerPosAndVelocityData, ShootDataGateway,
    };
    use crate::entities::Vec2;

    /// Trait to implement data marshalling when data crosses an architectural boundary
    pub trait Marshalling<To> {
        fn convert(&self) -> To;
    }

    /// Intermediate format for exchange with storage backend
    pub type Vec2Data = [f32; 2];
    impl Marshalling<Vec2> for Vec2Data {
        fn convert(&self) -> Vec2 {
            Vec2::new(self[0], self[1])
        }
    }
    impl Marshalling<Vec2Data> for Vec2 {
        fn convert(&self) -> Vec2Data {
            [self.get_x(), self.get_y()]
        }
    }

    /// Player Id
    pub type PlayerId = &'static str;
    /// Exchange format for Player Id
    pub type PlayerIdData = &'static str;
    impl Marshalling<&'static str> for &'static str {
        fn convert(&self) -> PlayerId {
            self
        }
    }

    pub type MissileId = usize;
    pub type MissileIdData = MissileId;
    impl Marshalling<usize> for usize {
        fn convert(&self) -> MissileId {
            *self
        }
    }

    /// Generic implementations for tuples (this could be done much more elegantly with a macro)
    impl<I1, O1> Marshalling<(O1,)> for (I1,)
    where
        I1: Marshalling<O1>,
    {
        fn convert(&self) -> (O1,) {
            (self.0.convert(),)
        }
    }
    impl<I1, I2, O1, O2> Marshalling<(O1, O2)> for (I1, I2)
    where
        I1: Marshalling<O1>,
        I2: Marshalling<O2>,
    {
        fn convert(&self) -> (O1, O2) {
            (self.0.convert(), self.1.convert())
        }
    }
    impl<I1, I2, I3, O1, O2, O3> Marshalling<(O1, O2, O3)> for (I1, I2, I3)
    where
        I1: Marshalling<O1>,
        I2: Marshalling<O2>,
        I3: Marshalling<O3>,
    {
        fn convert(&self) -> (O1, O2, O3) {
            (self.0.convert(), self.1.convert(), self.2.convert())
        }
    }
    impl<I1, I2, I3, I4, O1, O2, O3, O4> Marshalling<(O1, O2, O3, O4)> for (I1, I2, I3, I4)
    where
        I1: Marshalling<O1>,
        I2: Marshalling<O2>,
        I3: Marshalling<O3>,
        I4: Marshalling<O4>,
    {
        fn convert(&self) -> (O1, O2, O3, O4) {
            (
                self.0.convert(),
                self.1.convert(),
                self.2.convert(),
                self.3.convert(),
            )
        }
    }

    /// Generic implementation for vectors
    impl<I1, O1> Marshalling<Vec<O1>> for Vec<I1>
    where
        I1: Marshalling<O1> + Copy,
    {
        fn convert(&self) -> Vec<O1> {
            self.iter().map(|&item| item.convert()).collect()
        }
    }

    // Blanket implementation for Into implementers
    impl Marshalling<f32> for f32 {
        fn convert(&self) -> f32 {
            *self
        }
    }
}
