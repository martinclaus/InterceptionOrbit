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
        state.add_player("1");
        state.add_player("2");
        state
    }

    /// Add new player
    ///
    /// Reset player data to defaults, if player id already exists.
    fn add_player(&mut self, id: PlayerIdData) {
        self.player.insert(id, PlayerState::default());
    }

    // TODO: Propagate error
    fn get_player(&self, id: PlayerIdData) -> &PlayerState {
        self.player.get(id).expect("Player not found")
    }

    // TODO: Propagate error
    fn get_player_mut(&mut self, id: PlayerIdData) -> &mut PlayerState {
        self.player.get_mut(id).expect("Player not found")
    }

    fn add_missile(&mut self, player_id: PlayerIdData, data: MissileState) {
        self.get_player_mut(player_id).missiles.push(data);
    }

    fn get_missile(&self, player_id: PlayerIdData, missile_id: MissileIdData) -> &MissileState {
        &self.get_player(player_id).missiles[missile_id]
    }

    fn get_missile_mut(
        &mut self,
        player_id: PlayerIdData,
        missile_id: MissileIdData,
    ) -> &mut MissileState {
        &mut self.get_player_mut(player_id).missiles[missile_id]
    }

    /// Add a new Star
    fn add_star(&mut self, star: StarData) {
        self.stars.push(star);
    }
}

impl PlayerMovementDataGateway for RefCell<GameState> {
    fn get_player_orientation(&self, id: PlayerIdData) -> f32 {
        self.borrow().get_player(id).player_object.angle
    }

    fn set_player_orientation(&self, id: PlayerIdData, orientation: f32) {
        self.borrow_mut().get_player_mut(id).player_object.angle = orientation
    }

    fn get_player_acceleration(&self, id: PlayerIdData) -> Vec2Data {
        self.borrow().get_player(id).player_object.acceleration
    }

    fn set_player_acceleration(&self, id: PlayerIdData, acceleration: Vec2Data) {
        self.borrow_mut()
            .get_player_mut(id)
            .player_object
            .acceleration = acceleration
    }
}

impl ShootDataGateway for RefCell<GameState> {
    fn get_player_pos_and_velocity(&self, id: PlayerIdData) -> PlayerPosAndVelocityData {
        let state = self.borrow();
        let player = state.get_player(id);
        PlayerPosAndVelocityData {
            pos: player.player_object.position,
            angle: player.player_object.angle,
            velocity: player.player_object.velocity,
        }
    }

    fn get_player_missile_count(&self, id: PlayerIdData) -> usize {
        self.borrow().get_player(id).missiles.len()
    }

    fn create_missile_for_player(&self, id: PlayerIdData, missile: MissileLaunchData) {
        let missile = MovingObject {
            position: missile.pos,
            angle: missile.angle,
            velocity: missile.velocity,
            acceleration: Vec2Data::default(),
        };
        self.borrow_mut().add_missile(id, missile);
    }
}

impl GravityDataGateway for RefCell<GameState> {
    fn get_stars_position_and_mass(&self) -> Vec<StarData> {
        self.borrow().stars.clone()
    }

    fn get_player_pos_and_acc(&self) -> Vec<(PlayerIdData, Vec2Data, Vec2Data)> {
        let state = self.borrow();
        state
            .player
            .iter()
            .map(|(&id, p_state)| {
                (
                    id,
                    p_state.player_object.position,
                    p_state.player_object.acceleration,
                )
            })
            .collect()
    }

    fn get_missile_pos_and_acc(&self) -> Vec<(PlayerIdData, MissileIdData, Vec2Data, Vec2Data)> {
        let state = self.borrow();
        let mut result = Vec::new();
        state.player.iter().for_each(|(&pid, p_state)| {
            result.extend(
                p_state
                    .missiles
                    .iter()
                    .enumerate()
                    .map(|(mid, m_data)| (pid, mid, m_data.position, m_data.acceleration)),
            );
        });
        result
    }

    fn set_acceleration_for_player(&self, updates: Vec<(PlayerIdData, Vec2Data)>) {
        let mut state = self.borrow_mut();
        updates.into_iter().for_each(|(id, acceleration)| {
            state.get_player_mut(id).player_object.acceleration = acceleration
        });
    }

    fn set_acceleration_for_missiles(&self, updates: Vec<(PlayerIdData, MissileIdData, Vec2Data)>) {
        let mut state = self.borrow_mut();
        updates
            .into_iter()
            .for_each(|(player_id, missile_id, acceleration)| {
                state.get_missile_mut(player_id, missile_id).acceleration = acceleration
            });
    }
}
#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use crate::{
        physics::{GravityDataGateway, StarData},
        user_input::{MissileLaunchData, PlayerMovementDataGateway, ShootDataGateway},
    };

    use super::{GameState, MissileState, MovingObject};

    #[test]
    fn new_player_has_no_missiles() {
        let mut state = GameState::new();
        state.add_player("id");
        assert_eq!(state.get_player("id").missiles.len(), 0)
    }

    #[test]
    fn angle_updated_correctly() {
        let angle = 3.0;
        let state = RefCell::new(GameState::new());
        state.borrow_mut().add_player("id");
        state.set_player_orientation("id", angle);
        assert_eq!(state.get_player_orientation("id"), angle);
    }

    #[test]
    fn acceleration_set_correctly() {
        let acc = [3.0, 1000.0];
        let state = RefCell::new(GameState::new());
        state.borrow_mut().add_player("id");
        state.set_player_acceleration("id", acc);
        assert_eq!(state.get_player_acceleration("id"), acc);
    }

    #[test]
    fn player_pos_and_vel_retrieved_correctly() {
        let (pos, angle, vel) = ([0.0, 200.0], 4.0, [10.0, 4.0]);
        let state = RefCell::new(GameState::new());
        state.borrow_mut().add_player("id");
        state
            .borrow_mut()
            .get_player_mut("id")
            .player_object
            .position = pos;
        state.borrow_mut().get_player_mut("id").player_object.angle = angle;
        state
            .borrow_mut()
            .get_player_mut("id")
            .player_object
            .velocity = vel;

        let res = state.get_player_pos_and_velocity("id");
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
        state.borrow_mut().add_player("id");

        state.create_missile_for_player("id", data);

        let state_ref = state.borrow();
        assert_eq!(state_ref.get_player("id").missiles.len(), 1);
        let created_missile = state_ref.get_missile("id", 0);
        assert_eq!(created_missile.position, data.pos);
        assert_eq!(created_missile.angle, data.angle);
        assert_eq!(created_missile.velocity, data.velocity);
        assert_eq!(created_missile.acceleration, [0.0, 0.0]);
    }

    #[test]
    fn missiles_counted_correctly() {
        let state = RefCell::new(GameState::new());
        state.borrow_mut().add_player("id");
        for _ in 0..4 {
            state
                .borrow_mut()
                .add_missile("id", MissileState::default())
        }
        assert_eq!(state.get_player_missile_count("id"), 4)
    }

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
            let mut player1 = state_ref.get_player_mut("1");
            player1.player_object.position = [4.0, 1.0];
            player1.player_object.acceleration = [2.0, 1.0];
        }
        {
            let mut state_ref = state.borrow_mut();
            let mut player2 = state_ref.get_player_mut("2");
            player2.player_object.position = [1.0, 4.0];
            player2.player_object.acceleration = [1.0, 2.0];
        }

        let result = state.get_player_pos_and_acc();

        assert_eq!(result.len(), 2);
        for id_pos_acc in result {
            match id_pos_acc.0 {
                "1" => assert_eq!(id_pos_acc, ("1", [4.0, 1.0], [2.0, 1.0])),
                "2" => assert_eq!(id_pos_acc, ("2", [1.0, 4.0], [1.0, 2.0])),
                _ => panic!("Unexpected player id encountered"),
            }
        }
    }

    #[test]
    fn missile_pos_and_acc_correctly_returned() {
        let state = RefCell::new(GameState::new());
        state.borrow_mut().add_missile(
            "1",
            MovingObject {
                position: [0.0, 0.0],
                acceleration: [0.0, 0.0],
                ..MovingObject::default()
            },
        );
        state.borrow_mut().add_missile(
            "1",
            MovingObject {
                position: [1.0, 0.0],
                acceleration: [0.0, 1.0],
                ..MovingObject::default()
            },
        );
        state.borrow_mut().add_missile(
            "2",
            MovingObject {
                position: [0.0, 1.0],
                acceleration: [1.0, 0.0],
                ..MovingObject::default()
            },
        );

        let result = state.get_missile_pos_and_acc();

        assert_eq!(result.len(), 3);
        let mut have_seen = (false, false, false);
        for item in result {
            match item {
                ("1", 0, pos, acc) => {
                    assert_eq!((pos, acc), ([0.0, 0.0], [0.0, 0.0]));
                    have_seen.0 = true;
                }
                ("1", 1, pos, acc) => {
                    assert_eq!((pos, acc), ([1.0, 0.0], [0.0, 1.0]));
                    have_seen.1 = true;
                }
                ("2", 0, pos, acc) => {
                    assert_eq!((pos, acc), ([0.0, 1.0], [1.0, 0.0]));
                    have_seen.2 = true;
                }
                _ => panic!("Unexpected missile data encountered"),
            }
        }
        assert_eq!(have_seen, (true, true, true));
    }

    #[test]
    fn player_acc_correclty_updated() {
        let state = RefCell::new(GameState::new());

        state.set_acceleration_for_player(vec![("1", [20.0, 10.0]), ("2", [10.0, 20.0])]);
        state.set_acceleration_for_player(vec![("1", [5.0, 2.0])]);

        let state_ref = state.borrow();
        assert_eq!(
            state_ref.get_player("1").player_object.acceleration,
            [5.0, 2.0]
        );
        assert_eq!(
            state_ref.get_player("2").player_object.acceleration,
            [10.0, 20.0]
        );
    }

    #[test]
    fn missiles_correctly_updated() {
        let state = RefCell::new(GameState::new());
        state.borrow_mut().add_missile("1", MovingObject::default());
        state.borrow_mut().add_missile("1", MovingObject::default());
        state.borrow_mut().add_missile("2", MovingObject::default());

        state.set_acceleration_for_missiles(vec![
            ("1", 0, [20.0, 10.0]),
            ("1", 1, [40.0, 10.0]),
            ("2", 0, [10.0, 20.0]),
        ]);

        let state_ref = state.borrow();
        assert_eq!(state_ref.get_missile("1", 0).acceleration, [20.0, 10.0]);
        assert_eq!(state_ref.get_missile("1", 1).acceleration, [40.0, 10.0]);
        assert_eq!(state_ref.get_missile("2", 0).acceleration, [10.0, 20.0]);
    }
}
