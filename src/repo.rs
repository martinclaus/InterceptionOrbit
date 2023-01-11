//! Data repository implementations for in-memory and persistent storage.

use std::{cell::RefCell, collections::HashMap};

use super::repo_interfaces::*;

const MISSILE_CAPACITY: usize = 5;
const PLAYER_CAPACITY: usize = 2;

struct PlayerState {
    position: Vec2Data,
    angle: f32,
    velocity: Vec2Data,
    acceleration: Vec2Data,
    missiles: Vec<MissileLaunchData>,
}
// TODO: Set capacity of missile vectory according to Missile config
impl Default for PlayerState {
    fn default() -> Self {
        Self {
            position: Vec2Data::default(),
            angle: f32::default(),
            velocity: Vec2Data::default(),
            acceleration: Vec2Data::default(),
            missiles: Vec::with_capacity(MISSILE_CAPACITY),
        }
    }
}

pub struct GameState {
    player: HashMap<PlayerIdData, PlayerState>,
}
impl GameState {
    fn new() -> Self {
        let mut state = Self {
            player: HashMap::with_capacity(PLAYER_CAPACITY),
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
}

impl PlayerMovementDataGateway for RefCell<GameState> {
    fn get_player_orientation(&self, id: PlayerIdData) -> f32 {
        self.borrow().get_player(id).angle
    }

    fn set_player_orientation(&self, id: PlayerIdData, orientation: f32) {
        self.borrow_mut().get_player_mut(id).angle = orientation
    }

    fn get_player_acceleration(&self, id: PlayerIdData) -> Vec2Data {
        self.borrow().get_player(id).acceleration
    }

    fn set_player_acceleration(&self, id: PlayerIdData, acceleration: Vec2Data) {
        self.borrow_mut().get_player_mut(id).acceleration = acceleration
    }
}

impl ShootDataGateway for RefCell<GameState> {
    fn get_player_pos_and_velocity(&self, id: PlayerIdData) -> PlayerPosAndVelocityData {
        let state = self.borrow();
        let player = state.get_player(id);
        PlayerPosAndVelocityData {
            pos: player.position,
            angle: player.angle,
            velocity: player.velocity,
        }
    }

    fn get_player_missile_count(&self, id: PlayerIdData) -> usize {
        self.borrow().get_player(id).missiles.len()
    }

    fn create_missile_for_player(&self, id: PlayerIdData, missile: MissileLaunchData) {
        self.borrow_mut().get_player_mut(id).missiles.push(missile);
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use crate::user_input::{MissileLaunchData, PlayerMovementDataGateway, ShootDataGateway};

    use super::GameState;

    #[test]
    fn new_player_has_no_missiles() {
        let mut state = GameState::new();
        state.add_player("id");
        assert_eq!(state.player["id"].missiles.len(), 0)
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
        state.borrow_mut().get_player_mut("id").position = pos;
        state.borrow_mut().get_player_mut("id").angle = angle;
        state.borrow_mut().get_player_mut("id").velocity = vel;

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
        let player = state_ref.get_player("id");
        assert_eq!(player.missiles.len(), 1);
        assert_eq!(player.missiles[0], data)
    }

    #[test]
    fn missiles_counted_correctly() {
        let state = RefCell::new(GameState::new());
        state.borrow_mut().add_player("id");
        for _ in 0..4 {
            state
                .borrow_mut()
                .get_player_mut("id")
                .missiles
                .push(MissileLaunchData::default())
        }
        assert_eq!(state.get_player_missile_count("id"), 4)
    }
}
