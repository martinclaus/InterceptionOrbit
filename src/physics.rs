//! Physics engine

use std::rc::Rc;

use crate::{
    entities::{gravity, Vec2},
    repo_interfaces::{Marshalling, MissileId, MissileIdData, PlayerId, PlayerIdData, Vec2Data},
};

/// A star with a position and mass.
///
/// Stars excert gravitational attration to objects
#[derive(Clone, Copy, Debug)]
struct Star {
    pos: Vec2,
    mass: f32,
}

/// Data representation of a [`Star`] object
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StarData {
    pos: Vec2Data,
    mass: f32,
}
impl StarData {
    pub fn new(position: Vec2Data, mass: f32) -> Self {
        StarData {
            pos: position,
            mass: mass,
        }
    }
}
impl Marshalling<Star> for StarData {
    fn convert(&self) -> Star {
        Star {
            pos: self.pos.convert(),
            mass: self.mass,
        }
    }
}

/// Gravity use-case
///
/// This use case adds the acceleration by gravity to all player and missile objects
pub struct Gravity {
    repo: GravityDG,
}
impl Gravity {
    /// Create use case object
    pub fn new(repo: GravityDG) -> Self {
        Self { repo: repo.clone() }
    }

    /// Add gravitational acceleration to all player and missile objects
    pub fn execute(&self) {
        let stars = self.get_stars();
        self.apply_gravitation_to_players(&stars);
        self.apply_gravitation_to_missiles(&stars);
    }

    /// Get all attractants
    fn get_stars(&self) -> Vec<Star> {
        self.repo.get_stars_position_and_mass().convert()
    }

    /// Add gravitation acceleration to player objects
    fn apply_gravitation_to_players(&self, stars: &Vec<Star>) {
        let players = self.get_player_pos();
        let acc_updates = players
            .iter()
            .map(|&(id, player_pos, player_acc)| {
                (
                    id,
                    player_acc
                        + stars
                            .iter()
                            .map(|star| gravity(star.pos, star.mass, player_pos))
                            .sum(),
                )
            })
            .collect();
        self.set_acceleration_for_player(acc_updates)
    }

    /// Get position and acceleration of all player objects
    fn get_player_pos(&self) -> Vec<(PlayerId, Vec2, Vec2)> {
        self.repo.get_player_pos_and_acc().convert()
    }

    /// Update acceleration for player objects
    fn set_acceleration_for_player(&self, updates: Vec<(PlayerId, Vec2)>) {
        self.repo.set_acceleration_for_player(updates.convert())
    }

    /// Add gravitational acceleration to missile objects
    fn apply_gravitation_to_missiles(&self, stars: &[Star]) {
        let missiles = self.get_missile_pos_and_acc();
        let acc_updates = missiles
            .iter()
            .map(|&(player_id, missile_id, pos, acc)| {
                (
                    player_id,
                    missile_id,
                    acc + stars
                        .iter()
                        .map(|star| gravity(star.pos, star.mass, pos))
                        .sum(),
                )
            })
            .collect();
        self.set_acceleration_for_missiles(acc_updates)
    }

    /// Get position and acceleration of all missiles
    fn get_missile_pos_and_acc(&self) -> Vec<(PlayerId, MissileId, Vec2, Vec2)> {
        self.repo.get_missile_pos_and_acc().convert()
    }

    /// Update acceleration for missile objects
    fn set_acceleration_for_missiles(&self, updates: Vec<(PlayerId, MissileId, Vec2)>) {
        self.repo.set_acceleration_for_missiles(updates.convert())
    }
}

/// Data repository interface for gravity use case.
pub trait GravityDataGateway {
    /// Get all [`Star`] objects
    fn get_stars_position_and_mass(&self) -> Vec<StarData>;
    /// Get position and acceleration of all player objects
    fn get_player_pos_and_acc(&self) -> Vec<(PlayerIdData, Vec2Data, Vec2Data)>;
    /// Get position and acceleration of all miissile objects
    fn get_missile_pos_and_acc(&self) -> Vec<(PlayerIdData, MissileIdData, Vec2Data, Vec2Data)>;
    /// Update acceleration of player objects
    fn set_acceleration_for_player(&self, updates: Vec<(PlayerIdData, Vec2Data)>);
    /// Update acceleration of missile objects
    fn set_acceleration_for_missiles(&self, updates: Vec<(PlayerIdData, MissileIdData, Vec2Data)>);
}
type GravityDG = Rc<dyn GravityDataGateway>;

#[cfg(test)]
mod test_gravity {
    use super::{Gravity, GravityDataGateway, MissileIdData, PlayerIdData, StarData, Vec2Data};
    use std::{cell::RefCell, rc::Rc};

    #[derive(Default)]
    struct MockData {
        stars: Vec<StarData>,
        player_pos: Vec<(PlayerIdData, Vec2Data)>,
        player_acc: Vec<(PlayerIdData, Vec2Data)>,
        missile: Vec<(PlayerIdData, MissileIdData, Vec2Data, Vec2Data)>,
    }

    struct MockDataGateway {
        data: MockData,
    }
    impl GravityDataGateway for RefCell<MockDataGateway> {
        fn get_stars_position_and_mass(&self) -> Vec<StarData> {
            self.borrow().data.stars.clone()
        }

        fn get_player_pos_and_acc(&self) -> Vec<(PlayerIdData, Vec2Data, Vec2Data)> {
            let state = self.borrow();
            state
                .data
                .player_pos
                .iter()
                .map(|&(id, pos)| {
                    let index = state
                        .data
                        .player_acc
                        .iter()
                        .position(|&(id2, _)| id2 == id)
                        .unwrap();
                    (id, pos, state.data.player_acc[index].1)
                })
                .collect()
        }

        fn set_acceleration_for_player(&self, updates: Vec<(PlayerIdData, Vec2Data)>) {
            self.borrow_mut().data.player_acc = updates;
        }

        fn get_missile_pos_and_acc(
            &self,
        ) -> Vec<(PlayerIdData, MissileIdData, Vec2Data, Vec2Data)> {
            self.borrow().data.missile.clone()
        }

        fn set_acceleration_for_missiles(
            &self,
            updates: Vec<(PlayerIdData, MissileIdData, Vec2Data)>,
        ) {
            self.borrow_mut()
                .data
                .missile
                .iter_mut()
                .for_each(|missile| {
                    let pos = updates
                        .iter()
                        .position(|&item| (item.0 == missile.0) & (item.1 == missile.1))
                        .unwrap();
                    missile.3 = updates[pos].2;
                });
        }
    }

    fn setup_gravity_test(data: MockData) -> Rc<RefCell<MockDataGateway>> {
        Rc::new(RefCell::new(MockDataGateway { data: data }))
    }

    #[test]
    fn gravity_not_failing_when_repo_is_empty() {
        let repo = setup_gravity_test(MockData::default());
        let gravity = Gravity::new(repo.clone());
        gravity.execute();
    }

    #[test]
    fn gravity_applied_correctly_to_player() {
        let data = MockData {
            stars: vec![StarData {
                pos: [1.0, 0.0],
                mass: 1.0,
            }],
            player_pos: vec![("1", [11.0, 0.0]), ("2", [1.0, -10.0])],
            player_acc: vec![("1", [0.0, 1.0]), ("2", [1.0, 4.0])],
            ..MockData::default()
        };
        let repo = setup_gravity_test(data);
        let gravity = Gravity::new(repo.clone());
        gravity.execute();
    }
}

type PlayerInfo = (PlayerId, Vec2, Vec2, Vec2);
type PlayerInfoData = (PlayerIdData, Vec2Data, Vec2Data, Vec2Data);
type PosVelAcc = (Vec2, Vec2, Vec2);
type MissileInfo = (PlayerId, MissileId, Vec2, Vec2, Vec2);
type MissileInfoData = (PlayerIdData, MissileIdData, Vec2Data, Vec2Data, Vec2Data);

/// Integrate use-case
///
/// Integrates position and velocity of all objects. This will also set the acceleration to zero.
pub struct Integrate {
    // delta_time: f32,
    repo: Rc<dyn IntegrateDataGateway>,
}
impl Integrate {
    /// Create a new integration use case
    pub fn new(repo: Rc<dyn IntegrateDataGateway>) -> Self {
        Self { repo: repo }
    }

    /// Run the use case
    pub fn execute(&self, delta_time: impl Into<f32>) {
        let dt = delta_time.into();
        self.integrate_player(dt);
        self.integrate_missiles(dt);
    }

    /// Integrate a vector in time using eulers method
    fn integrate(v: Vec2, inc: Vec2, delta_time: f32) -> Vec2 {
        v + inc * delta_time
    }
    /// Integrate position and velocity and set acceleration to zero    
    fn integrate_pos_vel_and_acc(
        &self,
        pos: Vec2,
        vel: Vec2,
        acc: Vec2,
        delta_time: f32,
    ) -> PosVelAcc {
        (
            Self::integrate(pos, vel, delta_time),
            Self::integrate(vel, acc, delta_time),
            Vec2::zero(),
        )
    }

    /// Integrate position and velocity of all player objects. Also set acceleration to zero.
    fn integrate_player(&self, delta_time: f32) {
        self.set_player_info(self.get_player_info().map(|(id, pos, vel, acc)| {
            let (pos, vel, acc) = self.integrate_pos_vel_and_acc(pos, vel, acc, delta_time);
            (id, pos, vel, acc)
        }));
    }

    /// Iterator of position, velocity and acceleration of all player objects
    fn get_player_info(&self) -> impl Iterator<Item = PlayerInfo> {
        self.repo
            .get_player_info()
            .into_iter()
            .map(|item| item.convert())
    }
    /// Update position, velocity and acceleration for all player objects
    fn set_player_info(&self, data: impl Iterator<Item = PlayerInfo>) {
        self.repo
            .set_player_info(data.map(|item| item.convert()).collect())
    }

    /// Integrate position and velocity of all missile objects. Also set acceleration to zero.
    fn integrate_missiles(&self, delta_time: f32) {
        self.set_missile_info(self.get_missile_info().map(|(pid, mid, pos, vel, acc)| {
            let (pos, vel, acc) = self.integrate_pos_vel_and_acc(pos, vel, acc, delta_time);
            (pid, mid, pos, vel, acc)
        }));
    }

    /// Get position, velocity and acceleration of all missile objects
    fn get_missile_info(&self) -> impl Iterator<Item = MissileInfo> {
        self.repo
            .get_missile_info()
            .into_iter()
            .map(|item| item.convert())
    }
    /// Update position, velocity and acceleration for all missile objects
    fn set_missile_info(&self, data: impl Iterator<Item = MissileInfo>) {
        self.repo
            .set_missile_info(data.map(|item| item.convert()).collect());
    }
}

pub trait IntegrateDataGateway {
    /// Return `(id, position, velocity, acceleration)` for all player
    fn get_player_info(&self) -> Vec<PlayerInfoData>;
    /// Return `(player_id, missile_id, position, velocity, acceleration)` for all missiles
    fn get_missile_info(&self) -> Vec<MissileInfoData>;
    /// Update position, velocity and acceleration for all player
    fn set_player_info(&self, data: Vec<PlayerInfoData>);
    /// Update position, velocity and acceleration for all missiles
    fn set_missile_info(&self, data: Vec<MissileInfoData>);
}

#[cfg(test)]
mod test_integrate {
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    use crate::repo_interfaces::{MissileIdData, PlayerIdData, Vec2Data};

    use super::Integrate;

    #[derive(Default)]
    struct MockData {
        player_info: HashMap<PlayerIdData, (Vec2Data, Vec2Data, Vec2Data)>,
        missile_info: HashMap<(PlayerIdData, MissileIdData), (Vec2Data, Vec2Data, Vec2Data)>,
    }

    struct MockDataGateway {
        data: MockData,
    }
    impl super::IntegrateDataGateway for RefCell<MockDataGateway> {
        fn get_player_info(&self) -> Vec<super::PlayerInfoData> {
            self.borrow()
                .data
                .player_info
                .iter()
                .map(|(&id, &data)| (id, data.0, data.1, data.2))
                .collect()
        }

        fn set_player_info(&self, data: Vec<super::PlayerInfoData>) {
            let mut repo = self.borrow_mut();
            data.iter().for_each(|&(id, new_pos, new_vel, new_acc)| {
                repo.data.player_info.entry(id).and_modify(|value| {
                    *value = (new_pos, new_vel, new_acc);
                });
            });
        }

        fn get_missile_info(&self) -> Vec<super::MissileInfoData> {
            self.borrow()
                .data
                .missile_info
                .iter()
                .map(|(&id, &data)| (id.0, id.1, data.0, data.1, data.2))
                .collect()
        }

        fn set_missile_info(&self, data: Vec<super::MissileInfoData>) {
            let mut repo = self.borrow_mut();
            data.iter()
                .for_each(|&(pid, mid, new_pos, new_vel, new_acc)| {
                    repo.data
                        .missile_info
                        .entry((pid, mid))
                        .and_modify(|value| *value = (new_pos, new_vel, new_acc));
                });
        }
    }

    fn setup_integrate_test(data: MockData) -> Rc<RefCell<MockDataGateway>> {
        Rc::new(RefCell::new(MockDataGateway { data }))
    }

    #[test]
    fn integrate_works_if_no_players_are_present() {
        let gateway = setup_integrate_test(MockData::default());
        let integrate = Integrate::new(gateway.clone());
        integrate.execute(1.0);
        assert!(gateway.borrow().data.player_info.is_empty())
    }

    #[test]
    fn integrate_updates_player_pos_correctly() {
        let data = MockData {
            player_info: [
                ("0", ([1.0, 0.0], [1.0, 1.0], [0.0, 0.0])),
                ("1", ([1.0, 0.0], [0.0, 1.0], [0.0, 0.0])),
            ]
            .into(),
            ..MockData::default()
        };
        let gateway = setup_integrate_test(data);
        let integrate = Integrate::new(gateway.clone());
        integrate.execute(2.0);
        let repo = gateway.borrow();
        assert_eq!(
            repo.data.player_info.clone().get("0").unwrap().0,
            [3.0, 2.0]
        );
        assert_eq!(
            repo.data.player_info.clone().get("1").unwrap().0,
            [1.0, 2.0]
        );
    }

    #[test]
    fn integrate_updates_player_vel_correctly() {
        let data = MockData {
            player_info: [
                ("0", ([0.0, 0.0], [1.0, 1.0], [1.0, 0.0])),
                ("1", ([0.0, 0.0], [0.0, 1.0], [1.0, 1.0])),
            ]
            .into(),
            ..MockData::default()
        };
        let gateway = setup_integrate_test(data);
        let integrate = Integrate::new(gateway.clone());
        integrate.execute(2.0);
        let repo = gateway.borrow();
        assert_eq!(
            repo.data.player_info.clone().get("0").unwrap().1,
            [3.0, 1.0]
        );
        assert_eq!(
            repo.data.player_info.clone().get("1").unwrap().1,
            [2.0, 3.0]
        );
    }

    #[test]
    fn integrate_set_player_acceleration_to_zero() {
        let data = MockData {
            player_info: [
                ("0", ([0.0, 0.0], [1.0, 1.0], [1.0, 0.0])),
                ("1", ([0.0, 0.0], [0.0, 1.0], [1.0, 1.0])),
            ]
            .into(),
            ..MockData::default()
        };
        let gateway = setup_integrate_test(data);
        let integrate = Integrate::new(gateway.clone());
        integrate.execute(2.0);
        let repo = gateway.borrow();
        assert_eq!(
            repo.data.player_info.clone().get("0").unwrap().2,
            [0.0, 0.0]
        );
        assert_eq!(
            repo.data.player_info.clone().get("1").unwrap().2,
            [0.0, 0.0]
        );
    }

    #[test]
    fn integrate_works_if_no_missiles_are_present() {
        let gateway = setup_integrate_test(MockData::default());
        let integrate = Integrate::new(gateway.clone());
        integrate.execute(1.0);
        assert!(gateway.borrow().data.missile_info.is_empty())
    }

    #[test]
    fn integrate_updates_missile_pos_correctly() {
        let data = MockData {
            missile_info: [
                (("0", 0), ([1.0, 0.0], [1.0, 1.0], [0.0, 0.0])),
                (("1", 0), ([1.0, 0.0], [0.0, 1.0], [0.0, 0.0])),
                (("1", 1), ([1.0, 0.0], [1.0, 1.0], [0.0, 0.0])),
            ]
            .into(),
            ..MockData::default()
        };
        let gateway = setup_integrate_test(data);
        let integrate = Integrate::new(gateway.clone());
        integrate.execute(2.0);
        let repo = gateway.borrow();
        assert_eq!(
            repo.data.missile_info.clone().get(&("0", 0)).unwrap().0,
            [3.0, 2.0]
        );
        assert_eq!(
            repo.data.missile_info.clone().get(&("1", 0)).unwrap().0,
            [1.0, 2.0]
        );
        assert_eq!(
            repo.data.missile_info.clone().get(&("1", 1)).unwrap().0,
            [3.0, 2.0]
        );
    }

    #[test]
    fn integrate_updates_missile_vel_correctly() {
        let data = MockData {
            missile_info: [
                (("0", 0), ([0.0, 0.0], [1.0, 1.0], [1.0, 0.0])),
                (("1", 0), ([0.0, 0.0], [0.0, 1.0], [1.0, 1.0])),
                (("1", 1), ([0.0, 0.0], [1.0, 1.0], [1.0, 0.0])),
            ]
            .into(),
            ..MockData::default()
        };
        let gateway = setup_integrate_test(data);
        let integrate = Integrate::new(gateway.clone());
        integrate.execute(2.0);
        let repo = gateway.borrow();
        assert_eq!(
            repo.data.missile_info.clone().get(&("0", 0)).unwrap().1,
            [3.0, 1.0]
        );
        assert_eq!(
            repo.data.missile_info.clone().get(&("1", 0)).unwrap().1,
            [2.0, 3.0]
        );
        assert_eq!(
            repo.data.missile_info.clone().get(&("1", 1)).unwrap().1,
            [3.0, 1.0]
        );
    }

    #[test]
    fn integrate_set_missile_acceleration_to_zero() {
        let data = MockData {
            missile_info: [
                (("0", 0), ([0.0, 0.0], [1.0, 1.0], [1.0, 0.0])),
                (("1", 0), ([0.0, 0.0], [0.0, 1.0], [1.0, 1.0])),
                (("1", 1), ([0.0, 0.0], [1.0, 1.0], [1.0, 0.0])),
            ]
            .into(),
            ..MockData::default()
        };
        let gateway = setup_integrate_test(data);
        let integrate = Integrate::new(gateway.clone());
        integrate.execute(2.0);
        let repo = gateway.borrow();
        assert_eq!(
            repo.data.missile_info.clone().get(&("0", 0)).unwrap().2,
            [0.0; 2]
        );
        assert_eq!(
            repo.data.missile_info.clone().get(&("1", 0)).unwrap().2,
            [0.0; 2]
        );
        assert_eq!(
            repo.data.missile_info.clone().get(&("1", 1)).unwrap().2,
            [0.0; 2]
        );
    }
}
