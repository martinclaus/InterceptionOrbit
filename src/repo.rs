//! Data repository implementations for in-memory and persistent storage.

use std::{cell::RefCell, collections::HashMap};

use super::repo_interfaces::*;

const MISSILE_CAPACITY: usize = 5;
const PLAYER_CAPACITY: usize = 2;

type MissileState = MovingObject;

#[derive(Clone, Debug, Default, PartialEq)]
struct MovingObject {
    position: Vec2Data,
    angle: f32,
    velocity: Vec2Data,
    acceleration: Vec2Data,
}
impl MovingObject {
    fn set_acceleration(&mut self, acceleration: Vec2Data) -> &mut Self {
        self.acceleration = acceleration;
        self
    }
    fn set_velocity(&mut self, velocity: Vec2Data) -> &mut Self {
        self.velocity = velocity;
        self
    }
    fn set_position(&mut self, position: Vec2Data) -> &mut Self {
        self.position = position;
        self
    }
    fn set_angle(&mut self, angle: f32) -> &mut Self {
        self.angle = angle;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
struct PlayerState {
    player_object: MovingObject,
    missiles: Vec<MissileState>,
}
// TODO: Set capacity of missile vector according to Missile config
impl Default for PlayerState {
    fn default() -> Self {
        Self {
            player_object: MovingObject::default(),
            missiles: Vec::with_capacity(MISSILE_CAPACITY),
        }
    }
}

pub struct GameState {
    stars: Vec<StarData>,
    player: HashMap<PlayerIdData, PlayerState>,
}
impl GameState {
    pub fn new() -> Self {
        let mut state = Self {
            player: HashMap::with_capacity(PLAYER_CAPACITY),
            stars: Vec::new(),
        };
        state.add_player(1);
        state.add_player(2);
        state
    }

    /// Add new player
    ///
    /// Reset player data to defaults, if player id already exists.
    fn add_player(&mut self, id: PlayerIdData) {
        self.player.insert(id, PlayerState::default());
    }

    // TODO: Propagate error
    fn get_player(&self, id: &PlayerIdData) -> &PlayerState {
        self.player.get(id).expect("Player not found")
    }

    // TODO: Propagate error
    fn get_player_mut(&mut self, id: &PlayerIdData) -> &mut PlayerState {
        self.player.get_mut(id).expect("Player not found")
    }

    fn add_missile(&mut self, player_id: &PlayerIdData, data: MissileState) {
        self.get_player_mut(player_id).missiles.push(data);
    }

    #[cfg(test)]
    fn get_missile(&self, player_id: &PlayerIdData, missile_id: MissileIdData) -> &MissileState {
        &self.get_player(player_id).missiles[missile_id]
    }

    fn get_missile_mut(
        &mut self,
        player_id: &PlayerIdData,
        missile_id: MissileIdData,
    ) -> &mut MissileState {
        &mut self.get_player_mut(player_id).missiles[missile_id]
    }

    fn iter_player(&self) -> impl Iterator<Item = (PlayerId, &PlayerState)> {
        self.player.iter().map(|(id, data)| (*id, data))
    }

    fn iter_missiles(&self) -> impl Iterator<Item = (PlayerId, MissileId, &MissileState)> {
        self.player.iter().flat_map(|(p_id, player)| {
            player
                .missiles
                .iter()
                .enumerate()
                .map(|(m_id, missile)| (*p_id, m_id, missile))
        })
    }

    /// Add a new Star
    #[cfg(test)]
    fn add_star(&mut self, star: StarData) {
        self.stars.push(star);
    }
}

impl PlayerMovementDataGateway for RefCell<GameState> {
    fn get_player_orientation(&self, id: &PlayerIdData) -> f32 {
        self.borrow().get_player(id).player_object.angle
    }

    fn set_player_orientation(&self, id: &PlayerIdData, orientation: f32) {
        self.borrow_mut()
            .get_player_mut(id)
            .player_object
            .set_angle(orientation);
    }

    fn get_player_acceleration(&self, id: &PlayerIdData) -> Vec2Data {
        self.borrow().get_player(id).player_object.acceleration
    }

    fn set_player_acceleration(&self, id: &PlayerIdData, acceleration: Vec2Data) {
        self.borrow_mut()
            .get_player_mut(id)
            .player_object
            .set_acceleration(acceleration);
    }
}

impl ShootDataGateway for RefCell<GameState> {
    fn get_player_pos_and_velocity(&self, id: &PlayerIdData) -> PlayerPosAndVelocityData {
        // let state = self.borrow();
        let MovingObject {
            position: pos,
            angle,
            velocity,
            acceleration: _,
        } = self.borrow().get_player(id).player_object;
        PlayerPosAndVelocityData {
            pos,
            angle,
            velocity,
        }
    }

    fn get_player_missile_count(&self, id: &PlayerIdData) -> usize {
        self.borrow().get_player(id).missiles.len()
    }

    fn create_missile_for_player(&self, id: &PlayerIdData, missile: MissileLaunchData) {
        let mut missile_obj = MovingObject::default();
        missile_obj
            .set_position(missile.pos)
            .set_angle(missile.angle)
            .set_velocity(missile.velocity);
        self.borrow_mut().add_missile(id, missile_obj);
    }
}

impl GravityDataGateway for RefCell<GameState> {
    fn get_stars_position_and_mass(&self) -> Vec<StarData> {
        self.borrow().stars.clone()
    }

    fn get_player_pos_and_acc(&self) -> Vec<(PlayerIdData, Vec2Data, Vec2Data)> {
        self.borrow()
            .iter_player()
            .map(|(id, p_state)| {
                (
                    id,
                    p_state.player_object.position,
                    p_state.player_object.acceleration,
                )
            })
            .collect()
    }

    fn get_missile_pos_and_acc(&self) -> Vec<(PlayerIdData, MissileIdData, Vec2Data, Vec2Data)> {
        self.borrow()
            .iter_missiles()
            .map(|(p_id, m_id, missile)| (p_id, m_id, missile.position, missile.acceleration))
            .collect()
    }

    fn set_acceleration_for_player(&self, updates: Vec<(PlayerIdData, Vec2Data)>) {
        let mut state = self.borrow_mut();
        updates.into_iter().for_each(|(id, acceleration)| {
            state
                .get_player_mut(&id)
                .player_object
                .set_acceleration(acceleration);
        });
    }

    fn set_acceleration_for_missiles(&self, updates: Vec<(PlayerIdData, MissileIdData, Vec2Data)>) {
        let mut state = self.borrow_mut();
        updates
            .into_iter()
            .for_each(|(player_id, missile_id, acceleration)| {
                state
                    .get_missile_mut(&player_id, missile_id)
                    .set_acceleration(acceleration);
            });
    }
}

impl IntegrateDataGateway for RefCell<GameState> {
    fn get_player_info(&self) -> Vec<(PlayerIdData, Vec2Data, Vec2Data, Vec2Data)> {
        self.borrow()
            .iter_player()
            .map(|(id, player)| {
                (
                    id,
                    player.player_object.position,
                    player.player_object.velocity,
                    player.player_object.acceleration,
                )
            })
            .collect()
    }

    fn get_missile_info(&self) -> Vec<(PlayerIdData, MissileIdData, Vec2Data, Vec2Data, Vec2Data)> {
        self.borrow()
            .iter_missiles()
            .map(|(p_id, m_id, missile)| {
                (
                    p_id,
                    m_id,
                    missile.position,
                    missile.velocity,
                    missile.acceleration,
                )
            })
            .collect()
    }

    fn set_player_info(&self, data: Vec<(PlayerIdData, Vec2Data, Vec2Data, Vec2Data)>) {
        let mut state = self.borrow_mut();
        for (p_id, pos, vel, acc) in data {
            state
                .get_player_mut(&p_id)
                .player_object
                .set_position(pos)
                .set_velocity(vel)
                .set_acceleration(acc);
        }
    }

    fn set_missile_info(
        &self,
        data: Vec<(PlayerIdData, MissileIdData, Vec2Data, Vec2Data, Vec2Data)>,
    ) {
        let mut state = self.borrow_mut();
        for (player_id, missile_id, pos, vel, acc) in data {
            state
                .get_missile_mut(&player_id, missile_id)
                .set_position(pos)
                .set_velocity(vel)
                .set_acceleration(acc);
        }
    }
}

impl InGameState for RefCell<GameState> {}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use crate::{
        physics::{GravityDataGateway, IntegrateDataGateway, StarData},
        user_input::{MissileLaunchData, PlayerMovementDataGateway, ShootDataGateway},
    };

    use super::{GameState, MissileState, MovingObject};

    #[test]
    fn new_player_has_no_missiles() {
        let mut state = GameState::new();
        state.add_player(0);
        assert_eq!(state.get_player(&0).missiles.len(), 0)
    }

    //////////////////////////
    // PlayerMovementDG impl
    //////////////////////////
    #[test]
    fn angle_updated_correctly() {
        let angle = 3.0;
        let state = RefCell::new(GameState::new());
        state.borrow_mut().add_player(0);
        state.set_player_orientation(&0, angle);
        assert_eq!(state.get_player_orientation(&0), angle);
    }

    #[test]
    fn acceleration_set_correctly() {
        let acc = [3.0, 1000.0];
        let state = RefCell::new(GameState::new());
        state.borrow_mut().add_player(0);
        state.set_player_acceleration(&0, acc);
        assert_eq!(state.get_player_acceleration(&0), acc);
    }

    //////////////////////////
    // ShootDG impl
    //////////////////////////
    #[test]
    fn player_pos_and_vel_retrieved_correctly() {
        let (pos, angle, vel) = ([0.0, 200.0], 4.0, [10.0, 4.0]);
        let state = RefCell::new(GameState::new());
        state.borrow_mut().add_player(0);
        state.borrow_mut().get_player_mut(&0).player_object.position = pos;
        state.borrow_mut().get_player_mut(&0).player_object.angle = angle;
        state.borrow_mut().get_player_mut(&0).player_object.velocity = vel;

        let res = state.get_player_pos_and_velocity(&0);
        assert_eq!(res.pos, pos);
        assert_eq!(res.angle, angle);
        assert_eq!(res.velocity, vel);
    }

    #[test]
    fn missile_correctly_created() {
        let data = MissileLaunchData {
            pos: [0.0, 4.0],
            angle: 2.0,
            velocity: [10.0, 40.0],
        };
        let state = RefCell::new(GameState::new());
        state.borrow_mut().add_player(0);

        state.create_missile_for_player(&0, data);

        let state_ref = state.borrow();
        assert_eq!(state_ref.get_player(&0).missiles.len(), 1);
        let created_missile = state_ref.get_missile(&0, 0);
        assert_eq!(created_missile.position, data.pos);
        assert_eq!(created_missile.angle, data.angle);
        assert_eq!(created_missile.velocity, data.velocity);
        assert_eq!(created_missile.acceleration, [0.0, 0.0]);
    }

    #[test]
    fn missiles_counted_correctly() {
        let state = RefCell::new(GameState::new());
        state.borrow_mut().add_player(0);
        for _ in 0..4 {
            state.borrow_mut().add_missile(&0, MissileState::default())
        }
        assert_eq!(state.get_player_missile_count(&0), 4)
    }

    //////////////////////////
    // GravityDG impl
    //////////////////////////
    #[test]
    fn stars_correctly_returned() {
        let state = RefCell::new(GameState::new());
        let star1 = StarData::new([1.0, 0.0], 1.0);
        let star2 = StarData::new([0.0, 1.0], 2.0);
        state.borrow_mut().add_star(star1);
        state.borrow_mut().add_star(star2);

        let stars = state.get_stars_position_and_mass();

        assert_eq!(stars.len(), 2);
        assert_eq!(stars[0], star1);
        assert_eq!(stars[1], star2);
    }

    #[test]
    fn player_pos_and_acc_corectly_returned() {
        let state = RefCell::new(GameState::new());
        {
            let mut state_ref = state.borrow_mut();
            state_ref
                .get_player_mut(&1)
                .player_object
                .set_position([4.0, 1.0])
                .set_acceleration([2.0, 1.0]);
            state_ref
                .get_player_mut(&2)
                .player_object
                .set_position([1.0, 4.0])
                .set_acceleration([1.0, 2.0]);
        }

        let result = state.get_player_pos_and_acc();

        assert_eq!(result.len(), 2);
        for id_pos_acc in result {
            match id_pos_acc.0 {
                1 => assert_eq!(id_pos_acc, (1, [4.0, 1.0], [2.0, 1.0])),
                2 => assert_eq!(id_pos_acc, (2, [1.0, 4.0], [1.0, 2.0])),
                _ => panic!("Unexpected player id encountered"),
            }
        }
    }

    #[test]
    fn missile_pos_and_acc_correctly_returned() {
        let state = RefCell::new(GameState::new());
        for (p_id, pos, acc) in [
            (1, [0.0, 0.0], [0.0, 0.0]),
            (1, [1.0, 0.0], [0.0, 1.0]),
            (2, [0.0, 1.0], [1.0, 0.0]),
        ] {
            state.borrow_mut().add_missile(
                &p_id,
                MovingObject::default()
                    .set_position(pos)
                    .set_acceleration(acc)
                    .to_owned(),
            );
        }

        let result = state.get_missile_pos_and_acc();

        assert_eq!(result.len(), 3);
        for item in result {
            match item {
                (1, 0, pos, acc) => {
                    assert_eq!((pos, acc), ([0.0, 0.0], [0.0, 0.0]));
                }
                (1, 1, pos, acc) => {
                    assert_eq!((pos, acc), ([1.0, 0.0], [0.0, 1.0]));
                }
                (2, 0, pos, acc) => {
                    assert_eq!((pos, acc), ([0.0, 1.0], [1.0, 0.0]));
                }
                _ => panic!("Unexpected missile data encountered"),
            }
        }
    }

    #[test]
    fn player_acc_correclty_updated() {
        let state = RefCell::new(GameState::new());

        state.set_acceleration_for_player(vec![(1, [20.0, 10.0]), (2, [10.0, 20.0])]);
        state.set_acceleration_for_player(vec![(1, [5.0, 2.0])]);

        let state_ref = state.borrow();
        assert_eq!(
            state_ref.get_player(&1).player_object.acceleration,
            [5.0, 2.0]
        );
        assert_eq!(
            state_ref.get_player(&2).player_object.acceleration,
            [10.0, 20.0]
        );
    }

    #[test]
    fn missiles_correctly_updated() {
        let state = RefCell::new(GameState::new());
        for p_id in &[1, 1, 2] {
            state
                .borrow_mut()
                .add_missile(p_id, MovingObject::default());
        }

        state.set_acceleration_for_missiles(vec![
            (1, 0, [20.0, 10.0]),
            (1, 1, [40.0, 10.0]),
            (2, 0, [10.0, 20.0]),
        ]);

        assert_eq!(state.borrow().get_missile(&1, 0).acceleration, [20.0, 10.0]);
        assert_eq!(state.borrow().get_missile(&1, 1).acceleration, [40.0, 10.0]);
        assert_eq!(state.borrow().get_missile(&2, 0).acceleration, [10.0, 20.0]);
    }

    //////////////////////////
    // IntegrateDG impl
    //////////////////////////
    #[test]
    fn player_pos_vel_and_acc_corectly_returned() {
        let state = RefCell::new(GameState::new());
        state
            .borrow_mut()
            .get_player_mut(&1)
            .player_object
            .set_position([4.0, 1.0])
            .set_velocity([4.0, 1.0])
            .set_acceleration([2.0, 1.0]);
        state
            .borrow_mut()
            .get_player_mut(&2)
            .player_object
            .set_position([1.0, 4.0])
            .set_velocity([1.0, 5.0])
            .set_acceleration([1.0, 2.0]);

        let result = <RefCell<GameState> as IntegrateDataGateway>::get_player_info(&state);

        assert_eq!(result.len(), 2);
        for id_pos_acc in result {
            match id_pos_acc.0 {
                1 => assert_eq!(id_pos_acc, (1, [4.0, 1.0], [4.0, 1.0], [2.0, 1.0])),
                2 => assert_eq!(id_pos_acc, (2, [1.0, 4.0], [1.0, 5.0], [1.0, 2.0])),
                _ => panic!("Unexpected player id encountered"),
            }
        }
    }

    #[test]
    fn missile_pos_vel_and_acc_correctly_returned() {
        let state = RefCell::new(GameState::new());
        {
            let mut state = state.borrow_mut();
            for (pid, y) in [(1, 1.0), (1, 2.0), (2, 3.0)] {
                state.add_missile(
                    &pid,
                    MovingObject::default().set_position([0.0, y]).to_owned(),
                );
            }
        }
        let result = <RefCell<GameState> as IntegrateDataGateway>::get_missile_info(&state);
        for (p_id, m_id, pos, _, _) in result {
            match (p_id, m_id) {
                (1, 0) => assert_eq!(pos, [0.0, 1.0]),
                (1, 1) => assert_eq!(pos, [0.0, 2.0]),
                (2, 0) => assert_eq!(pos, [0.0, 3.0]),
                _ => panic!("Unexpected player id or missile id encountered. Got ({p_id}, {m_id})"),
            }
        }
    }

    #[test]
    fn missile_pos_vel_and_acc_set_correctly() {
        let state = RefCell::new(GameState::new());
        {
            let mut state = state.borrow_mut();
            for pid in &[1, 1, 2] {
                state.add_missile(pid, MovingObject::default());
            }
        }
        state.set_missile_info(vec![
            (1, 0, [1.0, 2.0], [1.0, 1.0], [2.0, 2.0]),
            (2, 0, [2.0, 1.0], [2.0, 3.0], [4.0, 5.0]),
        ]);
        for p_id in [1, 2] {
            for (
                m_id,
                &MovingObject {
                    position,
                    velocity,
                    acceleration,
                    angle: _,
                },
            ) in state.borrow().get_player(&p_id).missiles.iter().enumerate()
            {
                match (p_id, m_id) {
                    (1, 0) => assert_eq!(
                        (position, velocity, acceleration),
                        ([1.0, 2.0], [1.0, 1.0], [2.0, 2.0])
                    ),
                    (1, 1) => assert_eq!(
                        (position, velocity, acceleration),
                        ([0.0, 0.0], [0.0, 0.0], [0.0, 0.0])
                    ),
                    (2, 0) => assert_eq!(
                        (position, velocity, acceleration),
                        ([2.0, 1.0], [2.0, 3.0], [4.0, 5.0])
                    ),
                    _ => {
                        panic!("Invalid player_id or missile_id encountered. Got ({p_id}, {m_id})")
                    }
                }
            }
        }
    }
}
