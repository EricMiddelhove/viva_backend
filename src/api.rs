use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use crate::user;
use crate::user::{Dealer, Player};

#[derive(Deserialize)]
pub(crate) struct Gameday {
  pub(crate) name: String,
  pub(crate) initial_player_credits: u64,
}

#[derive(Deserialize)]
pub struct RegisterUser {
  pub(crate) nickname: String,
  pub(crate) name: String,
  pub(crate) pin: u32,
}


#[derive(Deserialize, Serialize)]
pub struct User{
  pub(crate) _id: ObjectId,
  pub(crate) nickname: Option<String>,
  pub(crate) name: Option<String>,
  pub(crate) pin: u32,
  pub(crate) credits: Option<u64>,
  pub(crate) password: Option<String>,
  pub(crate) role: Roles,
}
#[derive(Deserialize, Serialize)]
enum Roles {
  Player,
  Dealer,
}
impl From<user::User> for User{
  fn from(value: user::User) -> Self {

    match value {
      user::User::Player(player) => {
        User {
          _id: player._id,
          nickname: player.nickname,
          name: player.name,
          pin: player.pin,
          credits: Some(player.credits),
          password: None,
          role: Roles::Player,
        }
      }
      user::User::Dealer(dealer) => {
        User {
          _id: dealer._id,
          nickname: None,
          name: Some(dealer.name),
          pin: dealer.pin,
          credits: None,
          password: Some(dealer.password),
          role: Roles::Dealer,
        }
      }
    }

  }
}

impl Into<user::User> for User{
  fn into(self) -> user::User {
    match self.role {
      Roles::Player => {
        user::User::Player(Player{
          name: self.name,
          nickname: self.nickname,
          _id: self._id,
          credits: self.credits.unwrap(),
          pin: self.pin,
        })
      }
      Roles::Dealer => {
        user::User::Dealer(Dealer{
          name: self.name.unwrap(),
          _id: self._id,
          pin: self.pin,
          password: self.password.unwrap(),
        })
      }
    }
  }
}