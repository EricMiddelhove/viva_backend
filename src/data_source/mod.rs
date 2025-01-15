use std::io::{Error, ErrorKind};
use actix_web::web::Data;
use mongodb::bson::{doc, Document};
use mongodb::{Client, Collection};
use crate::{DATABASE_IDENT, MONGO_URL};

pub struct DataSource {
  pub database_identifier: &'static str,
  pub collection_identifier: &'static str,
  pub mongo_uri: &'static str,
}

pub const GAMEDAYS: DataSource = DataSource {
  database_identifier: DATABASE_IDENT,
  collection_identifier: "gamedays",
  mongo_uri: MONGO_URL,
};

pub const PENDING_USERS: DataSource = DataSource {
  database_identifier: DATABASE_IDENT,
  collection_identifier: "pending_users",
  mongo_uri: MONGO_URL,
};

pub const ACTIVE_USERS: DataSource = DataSource {
  database_identifier: DATABASE_IDENT,
  collection_identifier: "active_users",
  mongo_uri: MONGO_URL,
};


impl DataSource {

  pub async fn get_new_db_client(&self) -> Result<mongodb::Client, Error> {

    let c = Client::with_uri_str(&self.mongo_uri).await;
    
    match c {
      Ok(c) => Ok(c),
      Err(e) => {Err(Error::new(ErrorKind::ConnectionRefused, format!("Database: {}", e)))},
    }

  }

}