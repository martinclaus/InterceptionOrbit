//! Use-cases related to user input
//!
//! Use-cases are:
//! -  Player movement
//! -  Missile launch

/// Interface for commands issued by player input
pub trait InputCommand {
    fn execute(&self);
}

// Reexport player movement API
pub use movement::{MoveCommandFactory, MoveConfig, MoveInstruction, PlayerMovementDataGateway};

// Reexport shoot API
pub use shooting::{
    MissileConfig, MissileLaunchData, PlayerPosAndVelocityData, ShootCommandFactory,
    ShootDataGateway,
};

mod movement {
    use super::InputCommand;
    use crate::{
        entities::{trim_angle, Vec2},
        repo_interfaces::{Marshalling, PlayerId, PlayerIdData, Vec2Data},
    };
    use std::rc::Rc;

    /// Configuration object for player movement
    #[derive(Copy, Clone, Debug)]
    pub struct MoveConfig {
        angle_per_frame: f32,
        acceleration: f32,
    }

    impl MoveConfig {
        /// Create a new configuration object for player movement
        ///
        /// `angle` is the angle of rotation in degree per frame to rotate.
        /// `acceleration` is the scalar aceleration in the direction in which the
        /// Player points.
        pub fn new(angle: impl Into<f32>, acceleration: impl Into<f32>) -> Self {
            MoveConfig {
                angle_per_frame: angle.into().to_radians(),
                acceleration: acceleration.into(),
            }
        }

        /// Change of player orientation per frame in radians
        pub fn get_angle_per_frame(&self) -> f32 {
            self.angle_per_frame
        }

        /// Scalar acceleration in direction of player orientation
        pub fn get_acceleration(&self) -> f32 {
            self.acceleration
        }
    }

    /// Possible commands for player movement
    pub enum MoveInstruction {
        RotateLeft,
        RotateRight,
        Accelerate,
    }

    /// Move command factory
    pub struct MoveCommandFactory {
        pub config: MoveConfig,
        pub repo: DataGateway,
    }

    impl MoveCommandFactory {
        pub fn new(config: MoveConfig, repo: DataGateway) -> Self {
            Self {
                config,
                repo: repo.clone(),
            }
        }
        pub fn make_move_command(
            &self,
            player_id: PlayerId,
            instruction: MoveInstruction,
        ) -> Box<dyn InputCommand> {
            Box::new(MoveCommand {
                player_id,
                instruction,
                config: self.config,
                repo: self.repo.clone(),
            })
        }
    }

    /// Concrete type for player movement commands
    struct MoveCommand {
        player_id: PlayerId,
        instruction: MoveInstruction,
        config: MoveConfig,
        repo: DataGateway,
    }

    impl InputCommand for MoveCommand {
        fn execute(&self) {
            match self.instruction {
                MoveInstruction::RotateLeft => self.player_rotate_left(),
                MoveInstruction::RotateRight => self.player_rotate_right(),
                MoveInstruction::Accelerate => self.player_accelerate(),
            }
        }
    }

    impl MoveCommand {
        /// Rotate player by angle
        fn player_rotate(&self, angle: f32) {
            let orientation = self.repo.get_player_orientation(self.player_id).convert();
            let new_orientation = trim_angle(orientation + angle);
            self.repo
                .set_player_orientation(self.player_id, new_orientation.convert());
        }

        /// Rotate player by fixed angle to the left
        fn player_rotate_left(&self) {
            let angle = self.config.angle_per_frame;
            self.player_rotate(angle);
        }

        /// Rotate player by fixed angle to the right
        fn player_rotate_right(&self) {
            let angle = -self.config.angle_per_frame;
            self.player_rotate(angle);
        }
        /// accelerate player in current diretion by fixed amount
        fn player_accelerate(&self) {
            let orientation = self.repo.get_player_orientation(self.player_id).convert();
            let acc = self.repo.get_player_acceleration(self.player_id).convert();
            let new_acc = Vec2::new(self.config.acceleration, 0.0).rotate(orientation) + acc;
            self.repo
                .set_player_acceleration(self.player_id, new_acc.convert())
        }
    }

    /// Interface for player data provider.
    ///
    /// Every storage backend for player data must implement this trait to be usable for providing game state data.
    pub trait PlayerMovementDataGateway {
        fn get_player_orientation(&self, id: PlayerIdData) -> f32;
        fn set_player_orientation(&self, id: PlayerIdData, orientation: f32);
        fn get_player_acceleration(&self, id: PlayerIdData) -> Vec2Data;
        fn set_player_acceleration(&self, id: PlayerIdData, acceleration: Vec2Data);
    }

    type DataGateway = Rc<dyn PlayerMovementDataGateway>;

    #[cfg(test)]
    mod test {
        use std::{cell::RefCell, rc::Rc};

        use crate::entities::{Vec2, PI};

        use super::{
            MoveCommandFactory, MoveConfig, MoveInstruction, PlayerIdData,
            PlayerMovementDataGateway, Vec2Data,
        };

        #[derive(Default)]
        struct MockData {
            vec: (PlayerIdData, String, Vec2Data),
            scalar: (PlayerIdData, String, f32),
        }

        struct MockDataGateway {
            data: MockData,
        }
        impl PlayerMovementDataGateway for RefCell<MockDataGateway> {
            fn get_player_orientation(&self, _id: super::PlayerIdData) -> f32 {
                PI / 2.0
            }
            fn set_player_orientation(&self, id: super::PlayerIdData, orientation: f32) {
                self.borrow_mut().data.scalar = (id, "orientation".into(), orientation);
            }
            fn get_player_acceleration(&self, _id: super::PlayerIdData) -> super::Vec2Data {
                [50.0, 100.0]
            }
            fn set_player_acceleration(
                &self,
                id: super::PlayerIdData,
                acceleration: super::Vec2Data,
            ) {
                self.borrow_mut().data.vec = (id, "acceleration".into(), acceleration);
            }
        }

        fn setup_move_test() -> (MoveConfig, MoveCommandFactory, Rc<RefCell<MockDataGateway>>) {
            let move_config: MoveConfig = MoveConfig::new(5_f32 * PI / 180.0, 100.0);
            let repo = Rc::new(RefCell::new(MockDataGateway {
                data: MockData::default(),
            }));
            let command_factory = MoveCommandFactory::new(move_config, repo.clone());
            (move_config, command_factory, repo)
        }

        #[test]
        fn player_rotates_left() {
            let (move_config, command_factory, repo) = setup_move_test();
            let before = repo.get_player_orientation("id");
            command_factory
                .make_move_command("id", MoveInstruction::RotateLeft)
                .execute();
            assert_eq!(
                repo.borrow().data.scalar,
                (
                    "id",
                    "orientation".into(),
                    before + move_config.get_angle_per_frame(),
                )
            );
        }

        #[test]
        fn player_rotate_right() {
            let (move_config, command_factory, repo) = setup_move_test();
            let before = repo.get_player_orientation("id");
            command_factory
                .make_move_command("id", MoveInstruction::RotateRight)
                .execute();
            assert_eq!(
                repo.borrow().data.scalar,
                (
                    "id",
                    "orientation".into(),
                    before - move_config.get_angle_per_frame(),
                )
            );
        }

        #[test]
        fn player_accelerate() {
            let (move_config, command_factory, repo) = setup_move_test();
            let before = repo.get_player_acceleration("id");
            let orientation = repo.get_player_orientation("id");
            command_factory
                .make_move_command("id", MoveInstruction::Accelerate)
                .execute();
            let acc = Vec2::new(move_config.get_acceleration(), 0.0).rotate(orientation);
            assert_eq!(
                repo.borrow().data.vec,
                (
                    "id",
                    "acceleration".into(),
                    [before[0] + acc.get_x(), before[1] + acc.get_y()],
                )
            );
        }
    }
}

mod shooting {
    use super::InputCommand;
    use crate::entities::Vec2;
    use crate::repo_interfaces::{Marshalling, PlayerId, PlayerIdData, Vec2Data};
    use std::rc::Rc;

    /// Position, orientation and velocity of the player object
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct ObjectPosAndVelocityData {
        pub pos: Vec2Data,
        pub angle: f32,
        pub velocity: Vec2Data,
    }

    /// Position, orientation and velocity of the player object
    #[derive(Clone, Copy)]
    struct ObjectPosAndVelocity {
        pos: Vec2,
        angle: f32,
        velocity: Vec2,
    }
    impl Marshalling<PlayerPosAndVelocity> for PlayerPosAndVelocityData {
        fn convert(&self) -> PlayerPosAndVelocity {
            PlayerPosAndVelocity {
                pos: self.pos.convert(),
                angle: self.angle.convert(),
                velocity: self.velocity.convert(),
            }
        }
    }
    impl Marshalling<PlayerPosAndVelocityData> for PlayerPosAndVelocity {
        fn convert(&self) -> PlayerPosAndVelocityData {
            PlayerPosAndVelocityData {
                pos: self.pos.convert(),
                angle: self.angle.convert(),
                velocity: self.velocity.convert(),
            }
        }
    }

    /// Position, orientation and velocity of an object
    pub type PlayerPosAndVelocityData = ObjectPosAndVelocityData;
    /// Position, orientation and velocity of an object
    type PlayerPosAndVelocity = ObjectPosAndVelocity;

    /// Position, orientation and velocity of a missile object
    pub type MissileLaunchData = PlayerPosAndVelocityData;
    /// Position, orientation and velocity of a missile object
    type MissileLaunch = PlayerPosAndVelocity;

    /// Missile related configuration
    #[derive(Clone, Copy)]
    pub struct MissileConfig {
        /// Maximum number of missile a player may have
        max: usize,
        /// Relative speed of missile when fired
        initial_speed: f32,
        /// Initial distance of missile to player object
        initial_distance: f32,
    }

    impl MissileConfig {
        /// Create a new missile config
        pub fn new(
            max_missile: usize,
            initial_speed: impl Into<f32>,
            initial_distance: impl Into<f32>,
        ) -> MissileConfig {
            MissileConfig {
                max: max_missile,
                initial_speed: initial_speed.into(),
                initial_distance: initial_distance.into(),
            }
        }

        /// Maximum number of missile a player may have
        pub fn get_max_missile(&self) -> usize {
            self.max
        }

        /// Relative speed of missile when fired
        pub fn get_initial_speed(&self) -> f32 {
            self.initial_speed
        }
    }

    /// Factory for shoot command use cases
    pub struct ShootCommandFactory {
        config: MissileConfig,
        repo: Rc<dyn ShootDataGateway>,
    }

    impl ShootCommandFactory {
        /// Create factory for shoot command use case
        pub fn new(config: MissileConfig, repo: DataGateway) -> ShootCommandFactory {
            ShootCommandFactory {
                config,
                repo: repo.clone(),
            }
        }

        /// Create a new shoot command use case
        pub fn make_shoot_command(&self, player_id: PlayerId) -> Box<dyn InputCommand> {
            Box::new(ShootCommand {
                config: self.config,
                player_id,
                repo: self.repo.clone(),
            })
        }
    }

    /// Shoot command use-case integrator
    struct ShootCommand {
        config: MissileConfig,
        player_id: PlayerId,
        repo: DataGateway,
    }

    impl InputCommand for ShootCommand {
        fn execute(&self) {
            self.shoot()
        }
    }

    impl ShootCommand {
        /// shoot command use case
        fn shoot(&self) {
            if self.player_can_shoot_missile(self.player_id) {
                self.create_missile_for_player(self.player_id);
            }
        }

        /// Create a new missile for a player
        fn create_missile_for_player(&self, player_id: PlayerId) {
            let player = self
                .repo
                .get_player_pos_and_velocity(player_id.convert())
                .convert();
            let missile_pos =
                player.pos + Vec2::new(self.config.initial_distance, 0.0).rotate(player.angle);
            let missile_vel =
                player.velocity + Vec2::new(self.config.initial_speed, 0.0).rotate(player.angle);
            let missile_angle = missile_vel.angle();
            let new_missile = MissileLaunch {
                pos: missile_pos,
                angle: missile_angle,
                velocity: missile_vel,
            };
            self.repo
                .create_missile_for_player(player_id.convert(), new_missile.convert());
        }

        /// Check if player can shoot more missiles
        fn player_can_shoot_missile(&self, player_id: PlayerId) -> bool {
            let current_missile = self.repo.get_player_missile_count(player_id.convert());
            current_missile < self.config.max
        }
    }

    /// Interface of data gateway for shoot use-case
    pub trait ShootDataGateway {
        /// Return position, orientation and velocity of player
        fn get_player_pos_and_velocity(&self, id: PlayerIdData) -> PlayerPosAndVelocityData;

        /// Return number of active missiles of a player
        fn get_player_missile_count(&self, id: PlayerIdData) -> usize;

        /// Safe missile for a player
        fn create_missile_for_player(&self, id: PlayerIdData, missile: MissileLaunchData);
    }

    type DataGateway = Rc<dyn ShootDataGateway>;

    #[cfg(test)]
    mod test {
        use std::{cell::RefCell, f32::EPSILON, rc::Rc};

        use crate::{
            entities::Vec2,
            repo_interfaces::{Marshalling, PlayerIdData},
        };

        use super::{
            MissileConfig, MissileLaunchData, PlayerPosAndVelocityData, ShootCommandFactory,
            ShootDataGateway,
        };

        #[derive(Default)]
        struct MockData {
            player: PlayerPosAndVelocityData,
            player_missiles: Vec<(PlayerIdData, MissileLaunchData)>,
        }

        impl Default for PlayerPosAndVelocityData {
            fn default() -> Self {
                Self {
                    pos: Default::default(),
                    angle: Default::default(),
                    velocity: Default::default(),
                }
            }
        }

        struct MockDataGateway {
            data: MockData,
        }
        impl ShootDataGateway for RefCell<MockDataGateway> {
            fn get_player_missile_count(&self, id: PlayerIdData) -> usize {
                self.borrow()
                    .data
                    .player_missiles
                    .iter()
                    .filter(|(m_id, _)| *m_id == id)
                    .count()
            }

            fn get_player_pos_and_velocity(
                &self,
                _: PlayerIdData,
            ) -> super::PlayerPosAndVelocityData {
                PlayerPosAndVelocityData {
                    ..self.borrow().data.player
                }
            }

            fn create_missile_for_player(
                &self,
                id: PlayerIdData,
                missile: super::MissileLaunchData,
            ) {
                self.borrow_mut().data.player_missiles.push((id, missile));
            }
        }

        fn setup_shoot_test(
            data: MockData,
        ) -> (
            MissileConfig,
            ShootCommandFactory,
            Rc<RefCell<MockDataGateway>>,
        ) {
            let config = MissileConfig::new(3, 100.0_f32, 500_f32);
            let repo = Rc::new(RefCell::new(MockDataGateway { data: data }));
            let factory = ShootCommandFactory::new(config, repo.clone());
            (config, factory, repo)
        }

        #[test]
        fn missile_shot_when_max_is_not_reached() {
            let (config, command_factory, repo) = setup_shoot_test(MockData::default());
            let player_id = "id";
            let before_missile_count = repo.get_player_missile_count(player_id);
            assert_eq!(before_missile_count, 0);
            assert!(before_missile_count < config.get_max_missile());
            command_factory.make_shoot_command(player_id).execute();
            let after_missile_count = repo.get_player_missile_count(player_id);
            assert_eq!(after_missile_count, 1);
        }

        #[test]
        fn missile_not_shot_when_max_is_reached() {
            let (config, command_factory, repo) = setup_shoot_test(MockData::default());
            let player_id = "id";
            for _ in 0..config.max {
                repo.borrow_mut()
                    .data
                    .player_missiles
                    .push((player_id, MissileLaunchData::default()));
            }
            let before_missile_count = repo.get_player_missile_count(player_id);
            assert_eq!(before_missile_count, config.max);
            command_factory.make_shoot_command(player_id).execute();
            let after_missile_count = repo.borrow().data.player_missiles.len();
            assert_eq!(after_missile_count, before_missile_count);
        }

        #[test]
        fn missile_created_at_correct_location() {
            let player_pos = PlayerPosAndVelocityData {
                pos: [200.0, 100.0],
                angle: 0.56,
                velocity: [4.0, 45.0],
            };
            let (config, command_factory, repo) = setup_shoot_test(MockData {
                player: player_pos,
                ..MockData::default()
            });
            let player_id = "id";
            command_factory.make_shoot_command(player_id).execute();
            let expected_pos = repo.borrow().data.player.pos.convert()
                + Vec2::new(config.initial_distance, 0.0).rotate(repo.borrow().data.player.angle);
            assert!(
                (expected_pos - repo.borrow().data.player_missiles[0].1.pos.convert()).len()
                    < EPSILON
            );
        }

        #[test]
        fn missile_created_with_correct_velocity() {
            let player_pos = PlayerPosAndVelocityData {
                pos: [200.0, 100.0],
                angle: 0.56,
                velocity: [4.0, 45.0],
            };
            let (config, command_factory, repo) = setup_shoot_test(MockData {
                player: player_pos,
                ..MockData::default()
            });
            let player_id = "id";
            command_factory.make_shoot_command(player_id).execute();
            let expected_vel = repo.borrow().data.player.velocity.convert()
                + Vec2::new(config.initial_speed, 0.0).rotate(repo.borrow().data.player.angle);
            assert!(
                (expected_vel - repo.borrow().data.player_missiles[0].1.velocity.convert()).len()
                    < EPSILON
            );
        }

        #[test]
        fn missile_created_with_correct_angle() {
            let player_pos = PlayerPosAndVelocityData {
                pos: [200.0, 100.0],
                angle: 0.56,
                velocity: [4.0, 45.0],
            };
            let (config, command_factory, repo) = setup_shoot_test(MockData {
                player: player_pos,
                ..MockData::default()
            });
            let player_id = "id";
            command_factory.make_shoot_command(player_id).execute();
            let expected_vel = repo.borrow().data.player.velocity.convert()
                + Vec2::new(config.initial_speed, 0.0).rotate(repo.borrow().data.player.angle);
            let expected_angle = expected_vel.angle();
            assert!(
                (expected_angle - repo.borrow().data.player_missiles[0].1.angle).abs() < EPSILON
            );
        }
    }
}
