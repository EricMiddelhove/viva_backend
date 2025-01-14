use serde::{Deserialize, Serialize};
use crate::user::User;

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
  initial_costs: u32,
  active_players: Vec<User>,
  name: String,
  modification_history: Vec<Modification>
}

#[derive(Serialize, Deserialize, Debug)]
struct Modification {
  credit_change: u32,
  edited_player: User,
  editing_player: User,
}