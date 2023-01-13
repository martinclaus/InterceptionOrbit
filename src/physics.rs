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
#[derive(Clone, Copy, Debug)]
pub struct StarData {
    pos: [f32; 2],
    mass: f32,
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
    repo: DataGateway,
}
impl Gravity {
    /// Create use case object
    pub fn new(repo: DataGateway) -> Self {
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
type DataGateway = Rc<dyn GravityDataGateway>;

#[cfg(test)]
mod test {
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
