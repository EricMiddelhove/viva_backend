use std::io::ErrorKind;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use futures::TryStreamExt;
use mongodb::bson::{doc};
use mongodb::bson::oid::ObjectId;
use mongodb::Collection;
use mongodb::error::Error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::{data_source};
use crate::data_source::{DBUser, DataSource};

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
  pub(crate) name: Option<String>,
  pub(crate) nickname: Option<String>,
  pub(crate) _id: ObjectId,
  pub(crate) credits: u64,
  pub(crate) pin: u32,
  pub(crate) active_game: Option<ObjectId>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Dealer {
  pub(crate) name: String,
  pub(crate) _id: ObjectId,
  pub(crate) password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum User {
  Player(Player),
  Dealer(Dealer),
}

impl User {
  pub async fn new(data: User, data_source: DataSource) -> Result<ObjectId, Error> {
    let insert_doc: data_source::DBUser = data.into();

    let client = data_source.get_new_db_client().await?;
    let db = client.database(data_source.database_identifier);
    let coll: Collection<data_source::DBUser> = db.collection(data_source.collection_identifier.into());

    let res = coll.insert_one(&insert_doc).await?;

    res.inserted_id.as_object_id().ok_or_else(|| Error::from(ErrorKind::InvalidData))
  }

  pub async fn get(_id: ObjectId, data_source: DataSource) -> Result<Option<User>, Error> {
    let client = data_source.get_new_db_client().await?;

    let db = client.database(data_source.database_identifier);
    let collection: Collection<data_source::DBUser> = db.collection(&*data_source.collection_identifier.to_string());

    let filter = doc! { "_id": _id };
    let res = collection.find_one(filter).await;

    match res {
      Ok(Some(u)) => {
        let usr: User = u.into();
        Ok(Some(usr))
      }
      Ok(None) => Ok(None),
      Err(e) => Err(e),
    }
  }

  pub async fn get_by_role(role: data_source::Roles, data_source: DataSource) -> Result<Vec<DBUser>, Error> {
    let client = data_source.get_new_db_client().await?;

    let db = client.database(data_source.database_identifier);
    let collection: Collection<data_source::DBUser> = db.collection(&*data_source.collection_identifier.to_string());

    let filter = doc! { "role": role.to_string() };
    let res: Vec<DBUser> = collection.find(filter).await?.try_collect().await?;

    Ok(res)
  }

  pub async fn get_by_name(name: &str, data_source: DataSource) -> Result<Option<User>, Error> {
    let client = data_source.get_new_db_client().await?;

    let db = client.database(data_source.database_identifier);
    let collection: Collection<data_source::DBUser> = db.collection(&*data_source.collection_identifier.to_string());

    let filter = doc! { "name": name };
    let res = collection.find_one(filter).await;

    match res {
      Ok(Some(u)) => {
        let usr: User = u.into();
        Ok(Some(usr))
      }
      Ok(None) => Ok(None),
      Err(e) => Err(e),
    }
  }

  pub async fn join_game(user_id: ObjectId, game_id: Option<ObjectId>, pin: i64, user_data_source: DataSource) -> Result<bool, Error> {
    let client = user_data_source.get_new_db_client().await?;

    let db = client.database(user_data_source.database_identifier);
    let collection: Collection<User> = db.collection(&*user_data_source.collection_identifier.to_string());

    let filter = doc! {
      "_id": &user_id,
      "pin": &pin,
    };
    let modify = doc! { "$set": {"active_game": &game_id }  };

    let res = collection.update_one(filter, modify).await?;

    match res.matched_count {
      0 => Err(Error::from(ErrorKind::NotFound)),
      1 => Ok(true),
      _ => Ok(false),
    }
  }

  pub fn get_json_value(&self) -> serde_json::Value {
    match self {
      User::Player(u) => {
        json!(
          {
            "_id": u._id.to_string(),
            "nickname": u.nickname,
            "name": u.name,
            "credits": u.credits,
            "role": "player",
            "active_game": match u.active_game {
              None => {"".to_string()}
              Some(g) => {g.to_string()}
            }
          }
        )
      }
      User::Dealer(d) => {
        json!(
          {
            "_id": d._id.to_string(),
            "name": d.name,
            "role": "dealer",
          }
        )
      }
    }
  }
}

impl Dealer {
  pub async fn is_authenticated(&self, entered_password: &str) -> Result<bool, Error> {
    let password_hash = PasswordHash::parse(self.password.as_str(), Default::default());
    let hash = match password_hash {
      Ok(hash) => hash,
      Err(_) => {
        return Err(std::io::Error::new(ErrorKind::InvalidData, "Invalid password hash on db").into());
      }
    };

    let is_valid = Argon2::default()
      .verify_password(entered_password.as_bytes(), &hash)
      .is_ok();

    Ok(is_valid)
  }
}