use std::fmt;
use std::io::ErrorKind;
use mongodb::bson::{doc, Document};
use mongodb::bson::oid::ObjectId;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use rand::Rng;
use crate::api;
use crate::data_source::DataSource;

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
  pub(crate) name: Option<String>,
  pub(crate) nickname: Option<String>,
  pub(crate) _id: ObjectId,
  pub(crate) credits: u64,
  pub(crate) pin: u32
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Dealer {
  pub(crate) name: String,
  pub(crate) _id: ObjectId,
  pub(crate) pin: u32,
  pub(crate) password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum  User {
  Player(Player),
  Dealer(Dealer)
}

#[derive(Debug, Serialize, Deserialize)]
enum Roles {
  Admin,
  Player,
  Dealer,
}

type Error = std::io::Error;

impl User {
  pub async fn new(data: User, data_source: DataSource) -> Result<ObjectId, Error> {

    let insert_doc: api::User = data.into();

    let client = data_source.get_new_db_client().await?;
    let db = client.database(data_source.database_identifier);
    let coll: Collection<api::User> = db.collection(data_source.collection_identifier.into());

    let res = coll.insert_one(&insert_doc).await;

    match res {
      Ok(r) => { Ok(r.inserted_id.as_object_id().unwrap().into()) }
      Err(e) => { Err(std::io::Error::new(ErrorKind::ConnectionRefused, e.to_string())) }
    }
  }
}