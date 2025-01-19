use std::env;
use std::fmt::{Display, Formatter};
use std::io::{Error, ErrorKind};
use mongodb::Client;
use crate::{DATABASE_IDENT};

pub mod user;
pub mod gameday;
pub(crate) mod game;

pub struct DataSource {
  pub database_identifier: &'static str,
  pub collection_identifier: &'static str,
}

pub const GAMEDAYS: DataSource = DataSource {
  database_identifier: DATABASE_IDENT,
  collection_identifier: "gamedays",
};

pub const PENDING_USERS: DataSource = DataSource {
  database_identifier: DATABASE_IDENT,
  collection_identifier: "pending_users",
};

pub const ACTIVE_USERS: DataSource = DataSource {
  database_identifier: DATABASE_IDENT,
  collection_identifier: "active_users",
};

pub const GAMES: DataSource = DataSource {
  database_identifier: DATABASE_IDENT,
  collection_identifier: "games",
};


impl DataSource {
  pub async fn get_new_db_client(&self) -> Result<mongodb::Client, Error> {
    let mongo_uri = env::var("CUSTOMCONNSTR_MONGO_URI");

    let mongo_uri = match mongo_uri {
      Ok(mongo_uri) => { mongo_uri }

      Err(_) => {
        return Err(Error::new(ErrorKind::ConnectionRefused, "Mongo Connection url not set up - Env var should be: CUSTOMCONNSTR_MONGO_URI"));
      }
    };

    let c = Client::with_uri_str(mongo_uri).await;

    match c {
      Ok(c) => Ok(c),
      Err(e) => { Err(Error::new(ErrorKind::ConnectionRefused, format!("Database: {}", e))) }
    }
  }
}


use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use crate::data_source::user::{Dealer, Player};

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

#[derive(Deserialize)]
pub struct RegisterDealer {
  pub(crate) name: String,
  pub password: String,
}

#[derive(Deserialize)]
pub struct LoginDealer {
  pub(crate) name: String,
  pub password: String,
}


#[derive(Deserialize, Serialize)]
pub struct DBUser {
  pub(crate) _id: ObjectId,
  pub(crate) nickname: Option<String>,
  pub(crate) name: Option<String>,
  pub(crate) pin: u32,
  pub(crate) credits: Option<u64>,
  pub(crate) password: Option<String>,
  pub(crate) role: Roles,
  pub(crate) active_game: Option<ObjectId>,
}
#[derive(Deserialize, Serialize, Debug)]
pub enum Roles {
  Player,
  Dealer,
}

impl Display for Roles {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl From<user::User> for DBUser {
  fn from(value: user::User) -> Self {
    match value {
      user::User::Player(player) => {
        DBUser {
          _id: player._id,
          nickname: player.nickname,
          name: player.name,
          pin: player.pin,
          credits: Some(player.credits),
          password: None,
          role: Roles::Player,
          active_game: player.active_game,
        }
      }
      user::User::Dealer(dealer) => {
        DBUser {
          _id: dealer._id,
          nickname: None,
          name: Some(dealer.name),
          pin: 0,
          credits: None,
          password: Some(dealer.password),
          role: Roles::Dealer,
          active_game: None,
        }
      }
    }
  }
}

impl Into<user::User> for DBUser {
  fn into(self) -> user::User {
    match self.role {
      Roles::Player => {
        user::User::Player(self.into())
      }
      Roles::Dealer => {
        user::User::Dealer(Dealer {
          name: self.name.expect("Dealer has no name"),
          _id: self._id,
          password: self.password.expect("Dealer has no password"),
        })
      }
    }
  }
}

impl Into<Player> for DBUser {
  fn into(self) -> Player {
    Player {
      name: self.name,
      nickname: self.nickname,
      _id: self._id,
      credits: self.credits.expect("Players credits are none"),
      pin: self.pin,
      active_game: self.active_game,
    }
  }
}

#[derive(Deserialize, Serialize)]
pub struct Game {
  pub(crate) name: String,
  pub(crate) icon_id: String,
  pub(crate) join_fee: u64,
  pub description: String,
}