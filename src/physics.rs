//! Physics engine

use std::rc::Rc;

use crate::{
    entities::{gravity, Vec2},
    user_input::{PlayerId, PlayerIdData, Vec2Data},
};

type MissileId = usize;
pub type MissileIdData = MissileId;

#[derive(Clone, Copy, Debug)]
struct Star {
    pos: Vec2,
    mass: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct StarData {
    pos: [f32; 2],
    mass: f32,
}
impl From<StarData> for Star {
    fn from(value: StarData) -> Self {
        Star {
            pos: value.pos.into(),
            mass: value.mass,
        }
    }
}

pub struct Gravity {
    repo: DataGateway,
}
impl Gravity {
    pub fn new(repo: DataGateway) -> Self {
        Self { repo: repo.clone() }
    }

    pub fn execute(&self) {
        let stars = self.get_stars();
        self.apply_gravitation_to_players(&stars);
        self.apply_gravitation_to_missiles(&stars);
    }

    fn get_stars(&self) -> Vec<Star> {
        self.repo
            .get_stars_position_and_mass()
            .iter()
            .map(|&s| s.into())
            .collect()
    }

    fn apply_gravitation_to_players(&self, stars: &Vec<Star>) {
        let players = self.get_player_pos();
        let acc_updates = players.iter().map(|&(id, player_pos, player_acc)| {
            (
                id,
                player_acc
                    + stars
                        .iter()
                        .map(|star| gravity(star.pos, star.mass, player_pos))
                        .sum(),
            )
        });
        self.set_acceleration_for_player(acc_updates)
    }

    fn get_player_pos(&self) -> Vec<(PlayerId, Vec2, Vec2)> {
        self.repo
            .get_player_pos_and_acc()
            .iter()
            .map(|&(pid, pos, acc)| (pid.into(), pos.into(), acc.into()))
            .collect()
    }

    fn set_acceleration_for_player(&self, updates: impl Iterator<Item = (PlayerId, Vec2)>) {
        self.repo.set_acceleration_for_player(
            updates
                .map(|(id, acc)| (id.into(), acc.into()))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    }

    fn apply_gravitation_to_missiles(&self, stars: &[Star]) {
        let missiles = self.get_missile_pos_and_acc();
        let acc_updates = missiles.iter().map(|&(player_id, missile_id, pos, acc)| {
            (
                player_id,
                missile_id,
                acc + stars
                    .iter()
                    .map(|star| gravity(star.pos, star.mass, pos))
                    .sum(),
            )
        });
        self.set_acceleration_for_missiles(acc_updates)
    }

    fn get_missile_pos_and_acc(&self) -> Vec<(PlayerId, MissileId, Vec2, Vec2)> {
        self.repo
            .get_missile_pos_and_acc()
            .iter()
            .map(|&(player_id, missile_id, pos, acc)| {
                (player_id.into(), missile_id.into(), pos.into(), acc.into())
            })
            .collect::<Vec<_>>()
    }

    fn set_acceleration_for_missiles(
        &self,
        updates: impl Iterator<Item = (PlayerId, MissileId, Vec2)>,
    ) {
        self.repo.set_acceleration_for_missiles(
            updates
                .map(|(p_id, m_id, acc)| (p_id.into(), m_id.into(), acc.into()))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    }
}

pub trait GravityDataGateway {
    fn get_stars_position_and_mass(&self) -> Box<[StarData]>;
    fn get_player_pos_and_acc(&self) -> Box<[(PlayerIdData, Vec2Data, Vec2Data)]>;
    fn set_acceleration_for_player(&self, updates: Box<[(PlayerIdData, Vec2Data)]>);
    fn get_missile_pos_and_acc(&self) -> Box<[(PlayerIdData, MissileIdData, Vec2Data, Vec2Data)]>;
    fn set_acceleration_for_missiles(
        &self,
        updates: Box<[(PlayerIdData, MissileIdData, Vec2Data)]>,
    );
}
type DataGateway = Rc<dyn GravityDataGateway>;

#[cfg(test)]
mod test {
    use std::{cell::RefCell, rc::Rc};

    use crate::user_input::{PlayerIdData, Vec2Data};

    use super::{Gravity, GravityDataGateway, MissileIdData, StarData};

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
        fn get_stars_position_and_mass(&self) -> Box<[StarData]> {
            self.borrow().data.stars.clone().into_boxed_slice()
        }

        fn get_player_pos_and_acc(&self) -> Box<[(PlayerIdData, Vec2Data, Vec2Data)]> {
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

        fn set_acceleration_for_player(&self, updates: Box<[(PlayerIdData, Vec2Data)]>) {
            self.borrow_mut().data.player_acc = updates.into();
        }

        fn get_missile_pos_and_acc(
            &self,
        ) -> Box<[(PlayerIdData, MissileIdData, Vec2Data, Vec2Data)]> {
            self.borrow().data.missile.clone().into_boxed_slice()
        }

        fn set_acceleration_for_missiles(
            &self,
            updates: Box<[(PlayerIdData, MissileIdData, Vec2Data)]>,
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
