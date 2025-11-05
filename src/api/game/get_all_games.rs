use crate::data_source::{GAMES};
use actix_web::{
    get, HttpResponse, Responder,
};

use serde_json::{json, Value};
use crate::data_source::game::Game;

#[get("/game")]
async fn get_all_games() -> impl Responder {
    let res = Game::get_all(GAMES).await;

    let games = match res {
        Ok(g) => g,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    let out: Vec<Value> = games
        .into_iter()
        .map(|g| {
            json!(
                {
                    "_id": g._id.to_string(),
                    "join_fee": g.join_fee,
                    "name": g.name,
                    "icon_id": g.icon_id.to_string(),
                    "description": g.description
                }
            )
        })
        .collect();

    HttpResponse::Ok().json(out)
}
