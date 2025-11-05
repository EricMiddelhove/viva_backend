use actix_web::{get, web, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;

use crate::data_source::{self, user::User, DBUser, ACTIVE_USERS};

#[get("/user/{id}")]
async fn get_user(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();

    match id.as_str() {
        "player" => get_user_by_role(data_source::Roles::Player).await,
        "dealer" => get_user_by_role(data_source::Roles::Dealer).await,
        id => match ObjectId::parse_str(id) {
            Ok(id) => get_user_by_id(id).await,
            Err(e) => HttpResponse::NotFound().body(e.to_string()),
        },
    }
}

async fn get_user_by_role(role: data_source::Roles) -> HttpResponse {
    let users = User::get_by_role(role, ACTIVE_USERS).await;

    let users = match users {
        Ok(u) => u,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    let res = users.into_iter().map(DBUser::into).collect::<Vec<User>>();

    let res = res
        .into_iter()
        .map(|u: User| u.get_json_value())
        .collect::<Vec<serde_json::Value>>();

    HttpResponse::Ok().json(res)
}
async fn get_user_by_id(_id: ObjectId) -> HttpResponse {
    let user = User::get(_id, ACTIVE_USERS).await;

    match user {
        Ok(Some(user)) => {
            let usr = user.get_json_value();

            HttpResponse::Ok().json(usr)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        _ => HttpResponse::NotFound().body("User not found"),
    }
}
