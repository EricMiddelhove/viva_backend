use crate::api::is_user_authenticated_dealer;
use crate::data_source::gameday::Gameday;
use crate::data_source::user::{Player, User};
use crate::data_source::{GAMEDAYS, PENDING_USERS};
use crate::DATABASE_IDENT;
use actix_web::get;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::Collection;
use rand::Rng;
use serde_json::json;

#[get("/login_pin/{gameday_id}")]
async fn create_pending_user(path: web::Path<String>, req: HttpRequest) -> impl Responder {
    let dealer_id = req.headers().get("X-User-Id");
    let dealer_pw = req.headers().get("X-Dealer-Pw");

    let auth = is_user_authenticated_dealer(dealer_id, dealer_pw).await;
    match auth {
        Ok(_) => {}
        Err(err) => return err,
    }

    let client = GAMEDAYS.get_new_db_client().await;

    let client = match client {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let db = client.database(DATABASE_IDENT);
    let collection: Collection<Gameday> = db.collection(GAMEDAYS.collection_identifier);

    let gameday_id = path.into_inner();
    let gameday_id = ObjectId::parse_str(gameday_id.as_str());

    let gameday_id = match gameday_id {
        Ok(uuid) => uuid,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };

    let res = collection.find_one(doc! {"_id": gameday_id}).await;
    let gameday = match res {
        Ok(Some(gameday)) => gameday,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string());
        }
        _ => {
            return HttpResponse::NotFound().body("Gameday not found".to_string());
        }
    };

    let pin: u32 = rand::thread_rng().gen_range(100_000..999_999);

    let data = User::Player(Player {
        name: None,
        nickname: None,
        _id: ObjectId::new(),
        credits: gameday.initial_player_credits,
        pin,
        active_game: None,
    });

    let res = User::new(data, PENDING_USERS).await;

    let response: HttpResponse = match res {
        Ok(_) => {
            let body = json!({
                "pin": pin,
            });

            HttpResponse::Ok().json(body)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    };

    response
}
