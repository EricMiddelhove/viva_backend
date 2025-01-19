mod mongo_database_connector;
mod api;
mod data_source;

use actix_web::{delete, get, patch, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::dev::ResourcePath;
use actix_web::http::header::{HeaderValue};
use actix_web::middleware::Logger;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use futures::FutureExt;
use mongodb::Collection;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use rand::Rng;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::data_source::{DBUser, ACTIVE_USERS, GAMEDAYS, GAMES, PENDING_USERS};
use data_source::gameday::Gameday;
use data_source::user::{Player, User};
use data_source::game::Game;
use crate::data_source::user::Dealer;

const DATABASE_IDENT: &str = "viva_las_vegas";

#[post("/gameday")]
async fn create_gameday(body: web::Json<data_source::Gameday>) -> impl Responder {
  let client = GAMEDAYS.get_new_db_client().await;

  let client = match client {
    Ok(value) => { value }
    Err(err) => { return HttpResponse::InternalServerError().body(err.to_string()); }
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
    Err(err) => { HttpResponse::InternalServerError().body(err.to_string()) }
  }
}

#[get("/gameday")]
async fn get_gameday() -> impl Responder {
  let client = GAMEDAYS.get_new_db_client().await;

  let client = match client {
    Ok(value) => { value }
    Err(err) => { return HttpResponse::InternalServerError().body(err.to_string()); }
  };

  let db = client.database(DATABASE_IDENT);
  let collection: Collection<Gameday> = db.collection(GAMEDAYS.collection_identifier);

  let res = collection.find(Default::default()).await;
  match res {
    Ok(gameday) => {
      let gameday = gameday.deserialize_current();
      let gameday = match gameday {
        Ok(g) => { g }
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
    Err(err) => { return HttpResponse::InternalServerError().body(err.to_string()) }
  }
}

#[get("/login_pin/{gameday_id}")]
async fn create_pending_user(path: web::Path<String>, req: HttpRequest) -> impl Responder {
  let dealer_id = req.headers().get("X-User-Id");
  let dealer_pw = req.headers().get("X-Dealer-Pw");

  let auth = is_user_authenticated_dealer(dealer_id, dealer_pw).await;
  match auth {
    Ok(_) => {}
    Err(err) => { return err }
  }

  let client = GAMEDAYS.get_new_db_client().await;

  let client = match client {
    Ok(c) => { c }
    Err(e) => { return HttpResponse::InternalServerError().body(e.to_string()) }
  };


  let db = client.database(DATABASE_IDENT);
  let collection: Collection<Gameday> = db.collection(GAMEDAYS.collection_identifier);

  let gameday_id = path.into_inner();
  let gameday_id = ObjectId::parse_str(gameday_id.as_str());

  let gameday_id = match gameday_id {
    Ok(uuid) => { uuid }
    Err(err) => { return HttpResponse::InternalServerError().body(err.to_string()) }
  };

  let res = collection.find_one(doc! {"_id": gameday_id}).await;
  let gameday = match res {
    Ok(Some(gameday)) => {
      gameday
    }
    Err(e) => {
      return HttpResponse::InternalServerError().body(e.to_string());
    }
    _ => { return HttpResponse::NotFound().body("Gameday not found".to_string()); }
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
    Err(e) => { HttpResponse::InternalServerError().body(e.to_string()) }
  };

  response
}

#[post("/register")]
async fn register_user(body: web::Json<data_source::RegisterUser>) -> impl Responder {
  let client = PENDING_USERS.get_new_db_client().await;
  let client = match client {
    Ok(c) => { c }
    Err(e) => { return HttpResponse::InternalServerError().body(e.to_string()); }
  };

  let coll: Collection<data_source::DBUser> = client.database(DATABASE_IDENT).collection(PENDING_USERS.collection_identifier);

  let res = coll.find_one_and_delete(doc! {"pin": body.pin}).await;

  let user: User = match res {
    Ok(Some(user)) => {
      user
    }
    Err(e) => {
      return HttpResponse::InternalServerError().body(e.to_string());
    }
    _ => {
      let coll: Collection<data_source::DBUser> = client.database(DATABASE_IDENT).collection(ACTIVE_USERS.collection_identifier);

      let res = coll.find_one(doc! {"pin": body.pin, "name": body.name.as_str()}).await;

      match res {
        Ok(Some(u)) => {
          let json = json!({
            "_id": u._id.to_string(),
          });
          return HttpResponse::Ok().json(json);
        }
        Err(_) => {
          return HttpResponse::InternalServerError().body("user not found")
        }
        _ => {
          return HttpResponse::InternalServerError().body("user not found")
        }
      }
    }
  }.into();

  let client = ACTIVE_USERS.get_new_db_client().await;
  let client = match client {
    Ok(c) => { c }
    Err(e) => { return HttpResponse::InternalServerError().body(e.to_string()) }
  };

  let user = match user {
    User::Player(mut p) => {
      p.nickname = Some(body.nickname.clone());
      p.name = Some(body.name.clone());

      User::Player(p)
    }
    User::Dealer(_) => { return HttpResponse::InternalServerError().body("Recieved User::Dealer wher only User::Player was expected"); }
  };


  let db = client.database(DATABASE_IDENT);
  let collection: Collection<data_source::DBUser> = db.collection(ACTIVE_USERS.collection_identifier);

  let user: data_source::DBUser = user.into();

  let res = collection.insert_one(user).await;

  match res {
    Ok(r) => {
      let _id = r.inserted_id.as_object_id().expect("Expect inserted id to be valid ObjectId");


      let body = json!(
                {
                    "_id": _id.to_string(),
                }
            );
      HttpResponse::Ok().json(body)
    }
    Err(e) => { HttpResponse::InternalServerError().body(e.to_string()) }
  }
}

#[post("/game")]
async fn create_game(body: web::Json<data_source::Game>, req: HttpRequest) -> impl Responder {
  let dealer_id = req.headers().get("X-User-Id");
  let dealer_pw = req.headers().get("X-Dealer-Pw");

  let is_authorized = is_user_authenticated_dealer(dealer_id, dealer_pw).await;
  match is_authorized {
    Ok(_) => {}
    Err(res) => {
      return res;
    }
  };


  let res = Game::new(body.join_fee as u32, body.name.clone(), body.description.clone(), body.icon_id.clone(), GAMES).await;

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
    Err(e) => { HttpResponse::InternalServerError().body(e.to_string()) }
  }
}

#[get("/game")]
async fn get_all_games() -> impl Responder {
  let res = Game::get_all(GAMES).await;

  let games = match res {
    Ok(g) => { g }
    Err(e) => {
      return HttpResponse::InternalServerError().body(e.to_string());
    }
  };

  let out: Vec<Value> = games.into_iter().map(|g| {
    json!(
        {
            "_id": g._id.to_string(),
            "join_fee": g.join_fee,
            "name": g.name,
            "icon_id": g.icon_id.to_string(),
            "description": g.description
        }
    )
  }).collect();

  HttpResponse::Ok().json(out)
}

#[get("/game/{game_id}")]
async fn get_game(path: web::Path<String>) -> impl Responder {
  let id = path.into_inner();
  let _id = ObjectId::parse_str(id);
  let _id = match _id {
    Ok(id) => { id }
    Err(_) => { return HttpResponse::BadRequest().body("Invalid ID") }
  };


  let res = Game::get(&_id, GAMES).await;


  let users = Game::get_players(&_id, ACTIVE_USERS).await;

  let users = match users {
    Ok(v) => { v }
    Err(r) => {
      return HttpResponse::InternalServerError().body(r.to_string());
    }
  };

  match res {
    Ok(Some(game)) => {
      let body = json!(
                {
                    "_id": game._id.to_string(),
                    "initial_cost": 0,
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
    Err(e) => { HttpResponse::InternalServerError().body(e.to_string()) }
    _ => { HttpResponse::NotFound().body("Game not found".to_string()) }
  }
}

#[get("/game/{game_id}/{action}")]
async fn join_game(path: web::Path<(String, String)>, req: HttpRequest) -> impl Responder {
  let inner_path = path.into_inner();


  let id = inner_path.0;
  let game_id = ObjectId::parse_str(id);

  let game_id = match game_id {
    Ok(id) => { id }
    Err(_) => { return HttpResponse::BadRequest().body("Invalid Game ID") }
  };

  let id = req
    .headers()
    .get("X-User-Id");


  let user_id = match id {
    None => { return HttpResponse::BadRequest().body("User ID not provided in X-User-Id Header") }
    Some(id) => { id }
  };

  let user_id = match user_id.to_str() {
    Ok(id) => { id }
    Err(_) => { return HttpResponse::BadRequest().body("User ID is not valid utf-8") }
  };


  let user_id = match ObjectId::parse_str(user_id) {
    Ok(id) => { id }
    Err(_) => { return HttpResponse::BadRequest().body("Invalid USer ID") }
  };


  let to_join = match inner_path.1.as_str() {
    "join" => { Some(game_id) }
    "leave" => { None }
    _ => { return HttpResponse::BadRequest().body("Invalid Path") }
  };


  let pin = req.headers().get("X-User-Pin");
  let pin = match pin {
    Some(pin) => {
      match pin.to_str() {
        Ok(pin) => {
          match pin.parse::<i64>() {
            Ok(pin) => { pin }
            Err(_) => { return HttpResponse::BadRequest().body("Invalid Pin - no number") }
          }
        }
        Err(_) => { return HttpResponse::BadRequest().body("Invalid Pin - no number") }
      }
    }
    None => { return HttpResponse::BadRequest().body("No Pin provided in X-User-Pin Header") }
  };


  let res = User::join_game(user_id, to_join, pin, ACTIVE_USERS).await;

  match res {
    Ok(_) => { HttpResponse::Ok().body("success".to_string()) }
    Err(er) => {
      HttpResponse::InternalServerError().body(er.to_string())
    }
  }
}

#[patch("/game/{game_id}")]
async fn patch_game(path: web::Path<String>, body: web::Json<data_source::Game>, req: HttpRequest) -> impl Responder {
  let dealer_id = req.headers().get("X-User-Id");
  let dealer_pw = req.headers().get("X-Dealer-Pw");


  let auth = is_user_authenticated_dealer(dealer_id, dealer_pw).await;
  match auth {
    Ok(_) => {}
    Err(r) => {
      return r;
    }
  }

  let path = path.into_inner();
  let id = ObjectId::parse_str(path.as_str());

  let _id = match id {
    Ok(id) => { id }
    Err(e) => {
      return HttpResponse::InternalServerError().body(e.to_string())
    }
  };

  let body = body.into_inner();

  let res = Game::patch(&_id, GAMES, body).await;

  match res {
    Ok(Some(_)) => { HttpResponse::Ok().body("success".to_string()) }
    Ok(None) => { HttpResponse::NotFound().body("Game not found") }
    Err(er) => { HttpResponse::InternalServerError().body(er.to_string()) }
  }
}

#[delete("/game/{game_id}")]
async fn delete_game(path: web::Path<String>, req: HttpRequest) -> impl Responder {
  let dealer_id = req.headers().get("X-User-Id");
  let dealer_pw = req.headers().get("X-Dealer-Pw");


  let auth = is_user_authenticated_dealer(dealer_id, dealer_pw).await;
  match auth {
    Ok(_) => {}
    Err(r) => {
      return r;
    }
  }

  let id = ObjectId::parse_str(path.as_str());
  let _id = match id {
    Ok(id) => { id }
    Err(e) => {
      return HttpResponse::InternalServerError().body(e.to_string())
    }
  };

  let res = Game::delete(&_id, GAMES).await;

  match res {
    Ok(_) => { HttpResponse::Ok().body("success".to_string()) }
    Err(er) => { HttpResponse::InternalServerError().body(er.to_string()) }
  }
}

#[get("/user/{id}")]
async fn get_user(path: web::Path<String>) -> impl Responder {
  let id = path.into_inner();

  match id.as_str() {
    "player" => {
      get_user_by_role(data_source::Roles::Player).await
    }
    "dealer" => {
      get_user_by_role(data_source::Roles::Dealer).await
    }
    id => {
      match ObjectId::parse_str(id) {
        Ok(id) => {
          get_user_by_id(id).await
        }
        Err(e) => {
          HttpResponse::NotFound().body(e.to_string())
        }
      }
    }
  }
}

#[derive(Deserialize)]
struct CreditPatchBody {
  credits: i64,
}
async fn get_user_by_role(role: data_source::Roles) -> HttpResponse {
  let users = User::get_by_role(role, ACTIVE_USERS).await;

  let users = match users {
    Ok(u) => { u }
    Err(e) => {
      return HttpResponse::InternalServerError().body(e.to_string());
    }
  };

  let res = users.into_iter()
    .map(DBUser::into)
    .collect::<Vec<User>>();

  let res = res.into_iter()
    .map(|u: User| u.get_json_value())
    .collect::<Vec<serde_json::Value>>();

  HttpResponse::Ok().json(res)
}
async fn get_user_by_id(_id: ObjectId) -> HttpResponse {
  let user = User::get(_id, ACTIVE_USERS).await;

  println!("{:?}", user);
  match user {
    Ok(Some(user)) => {
      let usr = user.get_json_value();

      HttpResponse::Ok().json(usr)
    }
    Err(e) => { HttpResponse::InternalServerError().body(e.to_string()) }
    _ => {
      HttpResponse::NotFound().body("User not found")
    }
  }
}

#[post("/dealer/register")]
async fn register_dealer(body: web::Json<data_source::RegisterDealer>) -> impl Responder {
  let password = body.password.as_str();

  let salt: SaltString = SaltString::generate(&mut OsRng);
  let argon2: Argon2 = Argon2::default();

  let password_hash = argon2
    .hash_password(password.as_bytes(), &salt);

  let pw = match password_hash {
    Ok(pass) => pass,
    Err(e) => {
      return HttpResponse::BadRequest().body(e.to_string());
    }
  };

  let d = User::Dealer(Dealer {
    name: body.name.to_string(),
    _id: ObjectId::new(),
    password: pw.to_string(),
  });

  let r = User::new(d, ACTIVE_USERS).await;

  match r {
    Ok(_) => {
      HttpResponse::Ok().body("success".to_string())
    }
    Err(e) => {
      HttpResponse::InternalServerError().body(e.to_string())
    }
  }
}

#[post("/dealer/login")]
async fn login_dealer(body: web::Json<data_source::LoginDealer>) -> impl Responder {
  let password = body.password.as_str();
  let name = body.name.as_str();

  let usr = User::get_by_name(name, ACTIVE_USERS).await;

  let usr = match usr {
    Ok(u) => u,
    Err(e) => {
      return HttpResponse::InternalServerError().body(e.to_string());
    }
  };

  let usr = match usr {
    None => {
      return HttpResponse::NotFound().body("User not found")
    }
    Some(u) => { u }
  };

  match usr {
    User::Player(_) => {
      HttpResponse::Unauthorized().body("User is not a Dealer".to_string())
    }
    User::Dealer(u) => {
      let is_aut = u.is_authenticated(password).await;

      match is_aut {
        Ok(o) => {
          if o {
            return HttpResponse::Ok().finish();
          }

          HttpResponse::Unauthorized().body("Wrong Password".to_string())
        }
        Err(e) => {
          HttpResponse::InternalServerError().body(e.to_string())
        }
      }
    }
  }
}

#[get("/")]
async fn index() -> impl Responder {
  HttpResponse::PermanentRedirect().insert_header(("Location", "https://www.youtube.com/watch?v=R_ijlnDtKa4")).finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

  HttpServer::new(|| {
    App::new()
      .wrap(Logger::default())
      .service(index)
      .service(create_gameday)
      .service(create_pending_user)
      .service(register_user)
      .service(create_game)
      .service(get_game)
      .service(join_game)
      .service(get_user)
      .service(get_all_games)
      .service(register_dealer)
      .service(login_dealer)
      .service(get_gameday)
      .service(patch_game)
      .service(delete_game)
  })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

async fn is_user_authenticated_dealer(dealer_id: Option<&HeaderValue>, dealer_pw: Option<&HeaderValue>) -> Result<(), HttpResponse> {
  let dealer_id = match dealer_id {
    Some(id) => { id }
    None => {
      return Err(HttpResponse::Unauthorized().body("Dealer ID not provided in X-User-Id Header"));
    }
  };
  let dealer_id = match dealer_id.to_str() {
    Ok(d) => d,
    Err(x) => {
      return Err(HttpResponse::BadRequest().body(x.to_string()));
    }
  };

  let dealer_pw = match dealer_pw {
    None => { return Err(HttpResponse::Unauthorized().body("Dealer password not provided in X-Dealer-Pw Header")); }
    Some(x) => { x }
  };
  let dealer_pw = match dealer_pw.to_str() {
    Ok(d) => d,
    Err(x) => {
      return Err(HttpResponse::BadRequest().body(x.to_string()));
    }
  };

  let _id_dealer = ObjectId::parse_str(dealer_id);
  let _id_dealer = match _id_dealer {
    Ok(id) => { id }
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
    _ => { return Err(HttpResponse::NotFound().body("Dealer id not found")) }
  };

  let dealer = match user {
    User::Player(_) => {
      return Err(HttpResponse::Unauthorized().body("User Id does not refer to a dealer"));
    }
    User::Dealer(d) => d
  };

  let is_authorized = dealer.is_authenticated(dealer_pw).await;
  let is_authorized = match is_authorized {
    Ok(t) => { t }
    Err(_) => {
      return Err(HttpResponse::BadRequest().body("Password could not be authenticated"));
    }
  };

  if is_authorized {
    return Ok(());
  }

  Err(HttpResponse::Unauthorized().body("User is not authorized"))
}