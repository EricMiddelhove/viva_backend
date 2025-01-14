use std::io::Error;
use actix_web::web::Data;
use mongodb::Client;
use crate::MongoDatabaseConnector::DatabaseCollections::{Gamedays, Users};

#[derive(Debug)]
enum DatabaseCollections {
  Users{
    database_identifier: &'static str,
    collection_identifier: &'static str,
    mongo_uri: String,
  },
  Gamedays{
    database_identifier: &'static str,
    collection_identifier: &'static str,
    mongo_uri: String,
  },
}
impl DatabaseCollections::Users{
 async fn new(user_type: DatabaseCollections, database_identifier: &'static str, collection_identifier: &'static str, mongo_uri: String) -> Result<Self, Error>{

   let c = Client::with_uri_str(&mongo_uri)?;

   let db = c.database(database_identifier)?;

   let coll = db.collection(collection_identifier)?;

   Ok(Users{
     database_identifier,
     collection_identifier,
     mongo_uri
   })
 }
}

impl DatabaseCollections::Gamedays{
  fn new(database_identifier: &'static str, collection_identifier: &'static str, mongo_uri: Option<String>) -> Self{
    Gamedays{
      database_identifier,
      collection_identifier,
      mongo_uri
    }
  }
}