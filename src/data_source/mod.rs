use std::io::{Error, ErrorKind};
use mongodb::bson::{doc, Document};
use mongodb::{Client, Collection};

pub struct DataSource {
  pub mongo_uri: String,
}

impl DataSource {

  pub async fn get_new_db_client(&self) -> Result<mongodb::Client, Error> {

    let c = Client::with_uri_str(&self.mongo_uri).await;
    
    match c {
      Ok(c) => Ok(c),
      Err(e) => {Err(Error::new(ErrorKind::ConnectionRefused, format!("Database: {}", e)))},
    }

  }

}