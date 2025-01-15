use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::Collection;
use mongodb::error::Error;
use serde::{Deserialize, Serialize};
use crate::data_source;
use crate::data_source::{DBUser, DataSource};
use crate::data_source::user::{Player};
use futures::stream::{TryStreamExt};

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
  pub(crate) _id: ObjectId,
  pub(crate) description: String,
  pub(crate) initial_costs: u32,
  pub(crate) name: String,
}

impl Game {
  pub async fn new(initial_costs: u32, name: String, description: String, data_source: DataSource) -> Result<Self, Error> {
    let client = data_source.get_new_db_client().await?;
    let db = client.database(data_source.database_identifier);
    let collection = db.collection::<Game>(data_source.collection_identifier);

    let insert_doc = Game{
      _id: ObjectId::new(),
      description,
      initial_costs,
      name,
    };

    let res = collection.insert_one(&insert_doc).await;
    
    match res {
      Ok(_) => { Ok(insert_doc) }
      Err(e) => {Err(e)}
    }

  }

  pub async fn get(id: &ObjectId, data_source: DataSource) -> Result<Option<Self>, Error> {
    let client = data_source.get_new_db_client().await?;
    let db = client.database(data_source.database_identifier);
    let collection: Collection<Game> = db.collection(data_source.collection_identifier);

    let filter = doc! { "_id": id };

    let res = collection.find_one(filter).await?;
    Ok(res)
  }

  pub async fn get_players(game_id: &ObjectId, player_data_source: DataSource) -> Result<Vec<Player>, Error> {
    let client = player_data_source.get_new_db_client().await?;
    let db = client.database(player_data_source.database_identifier);
    let collection: Collection<data_source::DBUser> = db.collection(player_data_source.collection_identifier);

    let filter = doc! { "active_game": game_id };

    let res = collection.find(filter).await?;
    let res: Vec<DBUser> = res.try_collect().await?;

    let players: Vec<Player> = res.into_iter().map(DBUser::into).collect();

    Ok(players)
  }
}
