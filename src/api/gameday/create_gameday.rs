use actix_web::{post, web, HttpResponse, Responder};
use mongodb::Collection;
use serde_json::json;

use crate::{
    data_source::{self, gameday::Gameday, GAMEDAYS},
    DATABASE_IDENT,
};

#[post("/gameday")]
async fn create_gameday(body: web::Json<data_source::Gameday>) -> impl Responder {
    let client = GAMEDAYS.get_new_db_client().await;

    let client = match client {
        Ok(value) => value,
        Err(err) => {
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    };

    let db = client.database(DATABASE_IDENT);
    let collection: Collection<Gameday> = db.collection(GAMEDAYS.collection_identifier);

    let res = Gameday::new(body.initial_player_credits, body.name.clone(), &collection).await;

    match res {
        Ok(oid) => {
            let body = json!(
                {
                    "_id": oid.to_string(),
                }
            );

            HttpResponse::Ok().json(body)
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
