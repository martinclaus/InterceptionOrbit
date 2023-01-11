//! Domain rules for Interception Orbit game

mod entities;
pub mod repo;
pub mod user_input;

/// Reexport data gateway interfaces
pub mod repo_interfaces {
    pub use super::user_input::{
        MissileLaunchData, PlayerIdData, PlayerMovementDataGateway, PlayerPosAndVelocityData,
        ShootDataGateway, Vec2Data,
    };
}
