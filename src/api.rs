use serde::{Deserialize, Serialize};

struct ApiPlayer {
  name: String,
  nickname: String,
  credits: u64,
}


#[derive(Deserialize)]
pub(crate) struct Gameday {
  pub(crate) name: String,
  pub(crate) initial_credits: u32,
}

// impl From<User> for ApiPlayer {
//   fn from(value: User) -> Self {
//
//     Self {
//       name: value.to_string(),
//       nickname: value.to_string(),
//       credits: value.credits,
//     }
//
//   }
// }