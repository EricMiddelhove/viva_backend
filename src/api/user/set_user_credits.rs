use crate::api::is_user_authenticated_dealer;
use crate::data_source::user::User;
use crate::data_source::ACTIVE_USERS;
use actix_web::{patch, web, HttpRequest, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreditPatchBody {
    credits: i64,
}
#[patch("user/{user_id}/credits")]
pub async fn set_user_credits(
    path: web::Path<String>,
    body: web::Json<CreditPatchBody>,
    req: HttpRequest,
) -> impl Responder {
    let dealer_id = req.headers().get("X-User-Id");
    let dealer_pw = req.headers().get("X-Dealer-Pw");

    let auth = is_user_authenticated_dealer(dealer_id, dealer_pw).await;
    match auth {
        Ok(_) => {}
        Err(e) => {
            return e;
        }
    }

    let id = ObjectId::parse_str(path.as_str());
    let _id = match id {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    let res = User::set_credits(_id, body.credits, ACTIVE_USERS).await;

    match res {
        Ok(_) => HttpResponse::Ok().body("success".to_string()),
        Err(er) => HttpResponse::InternalServerError().body(er.to_string()),
    }
}
