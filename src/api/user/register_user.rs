use crate::data_source::user::User;
use crate::data_source::{self, ACTIVE_USERS, PENDING_USERS};
use crate::DATABASE_IDENT;
use actix_web::{post, web, HttpResponse, Responder};
use mongodb::bson::doc;
use mongodb::Collection;
use serde_json::json;

#[post("/register")]
async fn register_user(body: web::Json<data_source::RegisterUser>) -> impl Responder {
    let client = PENDING_USERS.get_new_db_client().await;
    let client = match client {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    let coll: Collection<data_source::DBUser> = client
        .database(DATABASE_IDENT)
        .collection(PENDING_USERS.collection_identifier);

    let user_api_response = match coll.find_one_and_delete(doc! {"pin": body.pin}).await {
        Ok(Some(user)) => user,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string());
        }
        Ok(None) => {
            let coll: Collection<data_source::DBUser> = client
                .database(DATABASE_IDENT)
                .collection(ACTIVE_USERS.collection_identifier);

            let res = coll.find_one(doc! {"pin": body.pin, "name": body.name.as_str(), "nickname": body.nickname.as_str()}).await;

            match res {
                Ok(Some(u)) => {
                    let json = json!({
                      "_id": u._id.to_string(),
                    });
                    return HttpResponse::Ok().json(json);
                }
                Err(_) => return HttpResponse::BadRequest().body("user not found"),
                _ => return HttpResponse::BadRequest().body("user not found"),
            }
        }
    };

    let user: User = user_api_response.into();

    let client = ACTIVE_USERS.get_new_db_client().await;
    let client = match client {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let user = match user {
        User::Player(mut p) => {
            p.nickname = Some(body.nickname.clone());
            p.name = Some(body.name.clone());

            User::Player(p)
        }
        User::Dealer(_) => {
            return HttpResponse::InternalServerError()
                .body("Recieved User::Dealer wher only User::Player was expected");
        }
    };

    let db = client.database(DATABASE_IDENT);
    let collection: Collection<data_source::DBUser> =
        db.collection(ACTIVE_USERS.collection_identifier);

    let user: data_source::DBUser = user.into();

    let res = collection.insert_one(user).await;

    match res {
        Ok(r) => {
            let _id = r
                .inserted_id
                .as_object_id()
                .expect("Expect inserted id to be valid ObjectId");

            let body = json!(
                {
                    "_id": _id.to_string(),
                }
            );
            HttpResponse::Ok().json(body)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
