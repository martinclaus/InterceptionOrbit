//! Use-cases related to user input
//!
//! Use-cases are:
//! -  Player movement
//! -  Missile launch

use crate::entities::Vec2;

/// A Player identifier
pub type PlayerId = &'static str;

/// Intermediate format for exchange with storage backend
type Vec2Data = [f32; 2];

impl From<Vec2Data> for Vec2 {
    fn from(value: Vec2Data) -> Self {
        Self::new(value[0], value[1])
    }
}

impl From<Vec2> for Vec2Data {
    fn from(value: Vec2) -> Self {
        [value.get_x(), value.get_y()]
    }
}

type PlayerIdData = PlayerId;

/// Interface for commands issued by player input
pub trait InputCommand {
    fn execute(&self);
}

// Reexport player movement API
pub use movement::{
    MoveCommandFactory, MoveConfig, MoveInstruction, PlayerMovementDataGateway,
    PlayerMovementDataGatewayFactory,
};

mod movement {
    use super::{InputCommand, PlayerId, PlayerIdData, Vec2Data};
    use crate::entities::{trim_angle, Vec2};
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
        pub repo: Rc<dyn PlayerMovementDataGateway>,
    }

    impl MoveCommandFactory {
        pub fn new(
            config: MoveConfig,
            repo_factory: impl PlayerMovementDataGateway + 'static,
        ) -> Self {
            Self {
                config,
                repo: Rc::new(repo_factory),
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

        #[cfg(test)]
        pub fn get_mock_repo(&self) -> Rc<dyn PlayerMovementDataGateway> {
            self.repo.clone()
        }
    }

    /// Concrete type for player movement commands
    struct MoveCommand {
        player_id: PlayerId,
        instruction: MoveInstruction,
        config: MoveConfig,
        repo: Rc<dyn PlayerMovementDataGateway>,
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
            let orientation = self.repo.get_player_orientation(self.player_id);
            let new_orientation = trim_angle(orientation + angle);
            self.repo
                .set_player_orientation(self.player_id, new_orientation);
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
            let orientation = self.repo.get_player_orientation(self.player_id);
            let acc = self.repo.get_player_acceleration(self.player_id).into();
            let new_acc = Vec2::new(self.config.acceleration, 0.0).rotate(orientation) + acc;
            self.repo
                .set_player_acceleration(self.player_id, new_acc.into())
        }
    }

    /// Abstrat fatory for player data gateway objects
    ///
    /// Storage backends must implement this interface. DIP
    pub trait PlayerMovementDataGatewayFactory {
        type Output: PlayerMovementDataGateway;
        fn make_gateway(&self) -> Self::Output;
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

    #[cfg(test)]
    mod test {
        use std::{cell::RefCell, rc::Rc};

        use crate::entities::{Vec2, PI};

        use super::{
            MoveCommandFactory, MoveConfig, MoveInstruction, PlayerIdData,
            PlayerMovementDataGateway, PlayerMovementDataGatewayFactory, Vec2Data,
        };

        struct MockDataGatewayFactory {
            data: Rc<MockData>,
        }
        impl PlayerMovementDataGatewayFactory for MockDataGatewayFactory {
            type Output = MockDataGateway;

            fn make_gateway(&self) -> Self::Output {
                MockDataGateway {
                    data: self.data.clone(),
                }
            }
        }

        #[derive(Default)]
        struct MockData {
            vec: RefCell<(PlayerIdData, String, Vec2Data)>,
            scalar: RefCell<(PlayerIdData, String, f32)>,
        }

        struct MockDataGateway {
            data: Rc<MockData>,
        }
        impl PlayerMovementDataGateway for MockDataGateway {
            fn get_player_orientation(&self, _id: super::PlayerIdData) -> f32 {
                PI / 2.0
            }
            fn set_player_orientation(&self, id: super::PlayerIdData, orientation: f32) {
                *self.data.scalar.borrow_mut() = (id, "orientation".into(), orientation);
            }
            fn get_player_acceleration(&self, _id: super::PlayerIdData) -> super::Vec2Data {
                [50.0, 100.0]
            }
            fn set_player_acceleration(
                &self,
                id: super::PlayerIdData,
                acceleration: super::Vec2Data,
            ) {
                *self.data.vec.borrow_mut() = (id, "acceleration".into(), acceleration);
            }
        }

        fn setup_move_test() -> (
            MoveConfig,
            MoveCommandFactory,
            Rc<MockData>,
            Rc<dyn PlayerMovementDataGateway>,
        ) {
            let move_config: MoveConfig = MoveConfig::new(5_f32 * PI / 180.0, 100.0);
            let mock_data = Rc::new(MockData::default());
            let repo = MockDataGatewayFactory {
                data: mock_data.clone(),
            }
            .make_gateway();
            let command_factory = MoveCommandFactory::new(move_config, repo);
            let repo = command_factory.get_mock_repo();
            (move_config, command_factory, mock_data, repo)
        }

        #[test]
        fn player_rotates_left() {
            let (move_config, command_factory, mock_data, repo) = setup_move_test();
            let before = repo.get_player_orientation("id");
            command_factory
                .make_move_command("id", MoveInstruction::RotateLeft)
                .execute();
            let changeset = mock_data.scalar.borrow().to_owned();
            assert_eq!(
                changeset,
                (
                    "id",
                    "orientation".into(),
                    before + move_config.get_angle_per_frame(),
                )
            );
        }

        #[test]
        fn player_rotate_right() {
            let (move_config, command_factory, mock_data, repo) = setup_move_test();
            let before = repo.get_player_orientation("id");
            command_factory
                .make_move_command("id", MoveInstruction::RotateRight)
                .execute();
            let changeset = mock_data.scalar.borrow().to_owned();
            assert_eq!(
                changeset,
                (
                    "id",
                    "orientation".into(),
                    before - move_config.get_angle_per_frame(),
                )
            );
        }

        #[test]
        fn player_accelerate() {
            let (move_config, command_factory, mock_data, repo) = setup_move_test();
            let before = repo.get_player_acceleration("id");
            let orientation = repo.get_player_orientation("id");
            command_factory
                .make_move_command("id", MoveInstruction::Accelerate)
                .execute();
            let acc = Vec2::new(move_config.get_acceleration(), 0.0).rotate(orientation);
            let changeset = mock_data.vec.borrow().to_owned();
            assert_eq!(
                changeset,
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
    use std::rc::Rc;

    use crate::entities::Vec2;

    use super::{InputCommand, PlayerId, PlayerIdData, Vec2Data};

    #[derive(Clone, Copy)]
    pub struct PlayerPosAndVelocityData {
        pos: Vec2Data,
        angle: f32,
        velocity: Vec2Data,
    }
    #[derive(Clone, Copy)]
    struct PlayerPosAndVelocity {
        pos: Vec2,
        angle: f32,
        velocity: Vec2,
    }
    impl From<PlayerPosAndVelocityData> for PlayerPosAndVelocity {
        fn from(value: PlayerPosAndVelocityData) -> Self {
            PlayerPosAndVelocity {
                pos: value.pos.into(),
                angle: value.angle.into(),
                velocity: value.velocity.into(),
            }
        }
    }
    impl From<PlayerPosAndVelocity> for PlayerPosAndVelocityData {
        fn from(value: PlayerPosAndVelocity) -> Self {
            PlayerPosAndVelocityData {
                pos: value.pos.into(),
                angle: value.angle.into(),
                velocity: value.velocity.into(),
            }
        }
    }

    pub type MissileLaunchData = PlayerPosAndVelocityData;
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

    pub struct ShootCommandFactory {
        config: MissileConfig,
        repo: Rc<dyn ShootDataGateway>,
    }

    impl ShootCommandFactory {
        pub fn new(
            config: MissileConfig,
            repo: impl ShootDataGateway + 'static,
        ) -> ShootCommandFactory {
            ShootCommandFactory {
                config,
                repo: Rc::new(repo),
            }
        }

        pub fn make_shoot_command(&self, player_id: PlayerId) -> Box<dyn InputCommand> {
            Box::new(ShootCommand {
                config: self.config,
                player_id,
                repo: self.repo.clone(),
            })
        }

        #[cfg(test)]
        pub fn get_mock_repo(&self) -> Rc<dyn ShootDataGateway> {
            self.repo.clone()
        }
    }

    struct ShootCommand {
        config: MissileConfig,
        player_id: PlayerId,
        repo: Rc<dyn ShootDataGateway>,
    }

    impl InputCommand for ShootCommand {
        fn execute(&self) {
            self.shoot()
        }
    }

    impl ShootCommand {
        fn shoot(&self) {
            if self.max_missile_reached_for_player(self.player_id) {
                return;
            }
            self.create_missile_for_player(self.player_id);
        }

        fn create_missile_for_player(&self, player_id: PlayerId) {
            let player: PlayerPosAndVelocity = self
                .repo
                .get_player_pos_and_velocity(player_id.into())
                .into();
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
                .create_missile_for_player(player_id.into(), new_missile.into());
        }

        fn max_missile_reached_for_player(&self, player_id: PlayerId) -> bool {
            let current_missile = self.repo.get_player_missile_count(player_id.into());
            current_missile >= self.config.max
        }
    }

    pub trait ShootDataGatewayFactory {
        type Output: ShootDataGateway;
        fn make_gateway(&self) -> Self::Output;
    }

    pub trait ShootDataGateway {
        fn get_player_pos_and_velocity(&self, id: PlayerIdData) -> PlayerPosAndVelocityData;

        fn get_player_missile_count(&self, id: PlayerIdData) -> usize;

        fn create_missile_for_player(&self, id: PlayerIdData, missile: MissileLaunchData);
    }

    #[cfg(test)]
    mod test {
        use std::{cell::RefCell, f32::EPSILON, rc::Rc};

        use crate::{
            entities::Vec2,
            user_input::{PlayerIdData, Vec2Data},
        };

        use super::{
            MissileConfig, MissileLaunchData, PlayerPosAndVelocityData, ShootCommandFactory,
            ShootDataGateway, ShootDataGatewayFactory,
        };

        struct MockDataGatewayFactory {
            data: Rc<MockData>,
        }
        impl ShootDataGatewayFactory for MockDataGatewayFactory {
            type Output = MockDataGateway;

            fn make_gateway(&self) -> Self::Output {
                MockDataGateway {
                    data: self.data.clone(),
                }
            }
        }

        #[derive(Default)]
        struct MockData {
            player: PlayerPosAndVelocityData,
            player_missiles: RefCell<Vec<(PlayerIdData, MissileLaunchData)>>,
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
            data: Rc<MockData>,
        }
        impl ShootDataGateway for MockDataGateway {
            fn get_player_missile_count(&self, id: PlayerIdData) -> usize {
                self.data
                    .player_missiles
                    .borrow()
                    .to_owned()
                    .iter()
                    .filter(|(m_id, _)| *m_id == id)
                    .count()
            }

            fn get_player_pos_and_velocity(
                &self,
                id: PlayerIdData,
            ) -> super::PlayerPosAndVelocityData {
                PlayerPosAndVelocityData { ..self.data.player }
            }

            fn create_missile_for_player(
                &self,
                id: PlayerIdData,
                missile: super::MissileLaunchData,
            ) {
                self.data.player_missiles.borrow_mut().push((id, missile));
            }
        }

        fn setup_shoot_test(
            data: MockData,
        ) -> (
            MissileConfig,
            ShootCommandFactory,
            Rc<MockData>,
            Rc<dyn ShootDataGateway>,
        ) {
            let config = MissileConfig::new(3, 100.0_f32, 500_f32);
            let data = Rc::new(MockData::default());
            let repo = MockDataGatewayFactory { data: data.clone() }.make_gateway();
            let factory = ShootCommandFactory::new(config, repo);
            let repo = factory.get_mock_repo();
            (config, factory, data, repo)
        }

        #[test]
        fn missile_shot_when_max_is_not_reached() {
            let (config, command_factory, data, repo) = setup_shoot_test(MockData::default());
            let player_id = "id";
            let before_missile_count = repo.get_player_missile_count(player_id);
            assert_eq!(before_missile_count, 0);
            assert!(before_missile_count < config.get_max_missile());
            command_factory.make_shoot_command(player_id).execute();
            let after_missile_count = data.player_missiles.borrow().to_owned().len();
            assert_eq!(after_missile_count, 1);
        }

        #[test]
        fn missile_not_shot_when_max_is_reached() {
            let (config, command_factory, data, repo) = setup_shoot_test(MockData::default());
            let player_id = "id";
            for _ in 0..config.max {
                data.player_missiles
                    .borrow_mut()
                    .push((player_id, MissileLaunchData::default()));
            }
            let before_missile_count = repo.get_player_missile_count(player_id);
            assert_eq!(before_missile_count, config.max);
            command_factory.make_shoot_command(player_id).execute();
            let after_missile_count = data.player_missiles.borrow().to_owned().len();
            assert_eq!(after_missile_count, before_missile_count);
        }

        #[test]
        fn missile_created_at_correct_location() {
            let player_pos = PlayerPosAndVelocityData {
                pos: [200.0, 100.0],
                angle: 0.56,
                velocity: [4.0, 45.0],
            };
            let (config, command_factory, data, repo) = setup_shoot_test(MockData {
                player: player_pos,
                ..MockData::default()
            });
            let player_id = "id";
            command_factory.make_shoot_command(player_id).execute();
            let expected_pos = <Vec2Data as Into<Vec2>>::into(data.player.pos)
                + Vec2::new(config.initial_distance, 0.0).rotate(data.player.angle);
            assert!(
                (expected_pos
                    - <Vec2Data as Into<Vec2>>::into(data.player_missiles.borrow()[0].1.pos))
                .len()
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
            let (config, command_factory, data, repo) = setup_shoot_test(MockData {
                player: player_pos,
                ..MockData::default()
            });
            let player_id = "id";
            command_factory.make_shoot_command(player_id).execute();
            let expected_vel = <Vec2Data as Into<Vec2>>::into(data.player.velocity)
                + Vec2::new(config.initial_speed, 0.0).rotate(data.player.angle);
            assert!(
                (expected_vel
                    - <Vec2Data as Into<Vec2>>::into(data.player_missiles.borrow()[0].1.velocity))
                .len()
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
            let (config, command_factory, data, repo) = setup_shoot_test(MockData {
                player: player_pos,
                ..MockData::default()
            });
            let player_id = "id";
            command_factory.make_shoot_command(player_id).execute();
            let expected_vel = <Vec2Data as Into<Vec2>>::into(data.player.velocity)
                + Vec2::new(config.initial_speed, 0.0).rotate(data.player.angle);
            let expected_angle = expected_vel.angle();
            assert!((expected_angle - data.player_missiles.borrow()[0].1.angle).abs() < EPSILON);
        }
    }
}
