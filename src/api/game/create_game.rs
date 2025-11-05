use crate::data_source::GAMES;
use crate::{api::is_user_authenticated_dealer, data_source::game::Game};
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use serde_json::json;

#[post("/game")]
async fn create_game(body: web::Json<Game>, req: HttpRequest) -> impl Responder {
    let dealer_id = req.headers().get("X-User-Id");
    let dealer_pw = req.headers().get("X-Dealer-Pw");

    let is_authorized = is_user_authenticated_dealer(dealer_id, dealer_pw).await;
    match is_authorized {
        Ok(_) => {}
        Err(res) => {
            return res;
        }
    };

    let res = Game::new(
        body.join_fee as u32,
        body.name.clone(),
        body.description.clone(),
        body.icon_id.clone(),
        GAMES,
    )
    .await;

    match res {
        Ok(game) => {
            let body = json!(
                {
                    "_id": game._id.to_string(),
                    "join_fee": game.join_fee,
                    "name": game.name,
                }
            );

            HttpResponse::Ok().json(body)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
