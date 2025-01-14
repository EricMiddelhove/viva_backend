use std::io::ErrorKind;
use mongodb::bson::{doc, Document};
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::data_source::DataSource;

#[derive(Debug, Serialize, Deserialize)]
pub struct  User {
  name: String,
  nickname: String,
  id: Uuid,
  credits: u64,
  pin: [u8; 6],
}

#[derive(Debug, Serialize, Deserialize)]
enum Roles {
  Admin,
  Player,
  Dealer,
}

type Error = std::io::Error;

// impl User {
//   fn read_info(&self, id: &str) -> Result<(), Error> {
//     let err = Error::new(ErrorKind::InvalidData, format!("User with id: {} not found", id));
//     Err(err)
//   }
//
//   async fn modify_credits(&mut self, player_id: &str, credit_difference: i64, collection: Collection<User>) -> Result<(), Error> {
//     match self.role {
//       Roles::Player => {
//         return Err(Error::new(ErrorKind::PermissionDenied, "You are not allowed to modify your credit"))
//       }
//       _ => {}
//     };
//
//     let filter = doc! { "_id": &player_id };
//
//     let modification = doc! { "credits": {"$add": credit_difference }};
//     let u = collection.update_one(filter, modification).await;
//
//     match u {
//       Ok(_) => Ok(()),
//       Err(err) => Err(Error::new(ErrorKind::Other, err.to_string())),
//     }
//   }
//
//   async fn retrieve_one(&self, id: &str, data_source: DataSource) -> Result<User, std::io::Error> {
//
//     let client = data_source.get_new_db_client();
//
//     let db = client.database(data_source.database_identifier);
//     let coll: Collection<User> = db.collection(data_source.collection_identifier);
//
//     let result = coll.find_one(doc! {"_id": &id}).await?;
//
//     match result {
//       None => Err(std::io::Error::new(ErrorKind::NotFound, "User not found")),
//       Some(some) => Ok(some),
//     }?
//   }
//
//   async fn new(data_source: DataSource, user: User) -> Result<User, std::io::Error> {
//     let client = data_source.get_new_db_client();
//
//     let db = client.database(data_source.database_identifier);
//     let coll: Collection<User> = db.collection(data_source.collection_identifier);
//
//     let document = doc! {
//       "name": user.name,
//       "nickname": user.nickname,
//       "_id": user.id,
//       "credits": user.credits,
//       ""
//     }
//
//     let result = coll.insert_one(document).await?;
//
//
//   }
// }