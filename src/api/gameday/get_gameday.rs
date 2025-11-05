use actix_web::{get, HttpResponse, Responder};
use mongodb::Collection;
use serde_json::json;

use crate::{
    data_source::{gameday::Gameday, GAMEDAYS},
    DATABASE_IDENT,
};

#[get("/gameday")]
async fn get_gameday() -> impl Responder {
    let client = GAMEDAYS.get_new_db_client().await;

    let client = match client {
        Ok(value) => value,
        Err(err) => {
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    };

    let db = client.database(DATABASE_IDENT);
    let collection: Collection<Gameday> = db.collection(GAMEDAYS.collection_identifier);

    let res = collection.find(Default::default()).await;
    match res {
        Ok(gameday) => {
            let gameday = gameday.deserialize_current();
            let gameday = match gameday {
                Ok(g) => g,
                Err(_) => {
                    return HttpResponse::InternalServerError().body("gameday not found");
                }
            };

            let body = json!({
              "_id": gameday._id.to_string(),
              "name": gameday.name,
              "initial_player_credits": gameday.initial_player_credits,
            });

            HttpResponse::Ok().json(body)
        }
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    }
}
