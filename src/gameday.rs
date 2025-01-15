use std::io::{Error, ErrorKind};
use mongodb::bson::{doc, Document, oid::ObjectId};
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use crate::game::Game;

#[derive(Serialize, Deserialize, Debug)]
pub struct Gameday {
  pub(crate) initial_player_credits: u64,
  pub(crate) name: String,
  games: Vec<Game>,
  _id: ObjectId,
}


impl Gameday {
  pub async fn new(initial_player_credits: u64, name: String, collection: &Collection<Gameday>) -> Result<ObjectId, Error> {

    let id = ObjectId::new();

    let insert_doc = Gameday{
      initial_player_credits,
      name,
      games: vec![],
      _id: id,
    };

    let res = collection.insert_one(&insert_doc).await;

    match res {
      Ok(_) => {Ok(id)}
      Err(e) => {Err(Error::new(ErrorKind::ConnectionRefused, e.to_string()))}
    }
  }

  pub async fn patch_one(id: ObjectId, modify_document: Document){

  }
}