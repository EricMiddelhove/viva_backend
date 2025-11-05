use crate::data_source::game::Game;
use crate::data_source::{ACTIVE_USERS, GAMES};
use actix_web::{get, web, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;
use serde_json::json;

#[get("/game/{game_id}")]
async fn get_game(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let _id = ObjectId::parse_str(id);
    let _id = match _id {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID"),
    };

    let res = Game::get(&_id, GAMES).await;

    let users = Game::get_active_players(&_id, ACTIVE_USERS).await;

    let users = match users {
        Ok(v) => v,
        Err(r) => {
            return HttpResponse::InternalServerError().body(r.to_string());
        }
    };

    match res {
        Ok(Some(game)) => {
            let body = json!(
                {
                    "_id": game._id.to_string(),
                    "join_fee": game.join_fee,
                    "icon_id": game.icon_id.to_string(),
                    "name": game.name,
                    "description": game.description,
                    "players": users.iter().map(|v| json!({
                        "name": v.name,
                        "nickname": v.nickname,
                        "_id": v._id.to_string(),
                        "credits": v.credits,
                    })).collect::<Vec<serde_json::Value>>(),
                }
            );

            HttpResponse::Ok().json(body)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        _ => HttpResponse::NotFound().body("Game not found".to_string()),
    }
}
