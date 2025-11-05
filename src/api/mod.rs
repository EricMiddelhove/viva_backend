pub mod game;
pub mod gameday;
pub mod user;

use actix_web::{http::header::HeaderValue, HttpResponse};
use mongodb::bson::oid::ObjectId;

use crate::data_source::{user::User, ACTIVE_USERS};

pub async fn is_user_authenticated_dealer(
    dealer_id: Option<&HeaderValue>,
    dealer_pw: Option<&HeaderValue>,
) -> Result<(), HttpResponse> {
    let dealer_id = match dealer_id {
        Some(id) => id,
        None => {
            return Err(
                HttpResponse::Unauthorized().body("Dealer ID not provided in X-User-Id Header")
            );
        }
    };
    let dealer_id = match dealer_id.to_str() {
        Ok(d) => d,
        Err(x) => {
            return Err(HttpResponse::BadRequest().body(x.to_string()));
        }
    };

    let dealer_pw = match dealer_pw {
        None => {
            return Err(HttpResponse::Unauthorized()
                .body("Dealer password not provided in X-Dealer-Pw Header"));
        }
        Some(x) => x,
    };
    let dealer_pw = match dealer_pw.to_str() {
        Ok(d) => d,
        Err(x) => {
            return Err(HttpResponse::BadRequest().body(x.to_string()));
        }
    };

    let _id_dealer = ObjectId::parse_str(dealer_id);
    let _id_dealer = match _id_dealer {
        Ok(id) => id,
        Err(_) => {
            return Err(HttpResponse::BadRequest().body("Dealer ID is not valid utf-8"));
        }
    };

    let user = User::get(_id_dealer, ACTIVE_USERS).await;
    let user = match user {
        Ok(Some(d)) => d,
        Err(e) => {
            return Err(HttpResponse::InternalServerError().body(e.to_string()));
        }
        _ => return Err(HttpResponse::NotFound().body("Dealer id not found")),
    };

    let dealer = match user {
        User::Player(_) => {
            return Err(HttpResponse::Unauthorized().body("User Id does not refer to a dealer"));
        }
        User::Dealer(d) => d,
    };

    let is_authorized = dealer.is_authenticated(dealer_pw).await;
    let is_authorized = match is_authorized {
        Ok(t) => t,
        Err(_) => {
            return Err(HttpResponse::BadRequest().body("Password could not be authenticated"));
        }
    };

    if is_authorized {
        return Ok(());
    }

    Err(HttpResponse::Unauthorized().body("User is not authorized"))
}
