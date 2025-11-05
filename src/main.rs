mod api;
mod data_source;

use crate::api::game::create_game::create_game;
use crate::api::game::get_all_games::get_all_games;
use crate::api::gameday::create_gameday::create_gameday;
use crate::api::gameday::get_gameday::get_gameday;
use crate::api::user::create_pending_user::create_pending_user;
use crate::api::user::get_user::get_user;
use crate::api::user::register_user::register_user;
use crate::api::user::set_user_credits::set_user_credits;
use crate::data_source::user::Dealer;
use crate::data_source::{DBUser, ACTIVE_USERS, GAMES, PENDING_USERS};
use actix_web::middleware::Logger;
use actix_web::{
    delete, get, patch, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use data_source::game::Game;
use data_source::user::{Player, User};
use log::info;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::{Collection, IndexModel};
use rand::Rng;
use serde::Serialize;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::{env, io};
use crate::api::game::get_game::get_game;
use crate::api::is_user_authenticated_dealer;

const DATABASE_IDENT: &str = "viva_las_vegas";

#[get("/game/{game_id}/leaderboard")]
async fn get_game_leaderboard(path: web::Path<String>) -> impl Responder {
    // Returns a list of players entered in the game. List remains unsorted. Sorting is up to the
    // client
    //
    #[derive(Serialize)]
    struct ExpandedPlayer {
        player_id: ObjectId,
        nickname: Option<String>,
        credit_delta: i64,
        total_credits: u64,
    }

    let id = path.into_inner();
    let game_id = match ObjectId::parse_str(id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid Game ID"),
    };

    let game = match Game::get(&game_id, GAMES).await {
        Ok(Some(g)) => g,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Internal Server Error when fetching game")
        }
        _ => return HttpResponse::BadRequest().body("Game not existent"),
    };

    let player_information = match game.player_information {
        Some(p) => p,
        None => return HttpResponse::Ok().body("No Leaderboard available for this game"),
    };

    let mut expanded_players = Vec::with_capacity(player_information.len());
    for p in player_information {
        let user = match User::get(p.player_id, ACTIVE_USERS).await {
            Ok(Some(u)) => u,
            Ok(None) => {
                return HttpResponse::NotFound().body("User not found");
            }
            Err(e) => {
                return HttpResponse::InternalServerError().body(e.to_string());
            }
        };
        let player: Player = match user.try_into() {
            Ok(p) => p,
            Err(_) => {
                return HttpResponse::BadRequest().body("User is not a player");
            }
        };

        let exp_user = ExpandedPlayer {
            player_id: p.player_id,
            nickname: player.nickname,
            credit_delta: p.player_credit_delta,
            total_credits: player.credits,
        };

        expanded_players.push(exp_user);
    }

    let body = json!({
        "players":  expanded_players.iter().map(|p| json!({
            "_id": p.player_id.to_string(),
            "nickname": p.nickname,
            "credit_delta": p.credit_delta,
            "total_credits": p.total_credits,
        })).collect::<Vec<serde_json::Value>>(),}
    );

    HttpResponse::Ok().json(body)
}

#[get("/game/{game_id}/{action}")]
async fn join_game(path: web::Path<(String, String)>, req: HttpRequest) -> impl Responder {
    let inner_path = path.into_inner();

    let id = inner_path.0;
    let game_id = ObjectId::parse_str(id);

    let game_id = match game_id {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid Game ID"),
    };

    let id = req.headers().get("X-User-Id");

    let user_id = match id {
        None => return HttpResponse::BadRequest().body("User ID not provided in X-User-Id Header"),
        Some(id) => id,
    };

    let user_id = match user_id.to_str() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("User ID is not valid utf-8"),
    };

    let user_id = match ObjectId::parse_str(user_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid USer ID"),
    };

    let to_join = match inner_path.1.as_str() {
        "join" => Some(game_id),
        "leave" => None,
        _ => return HttpResponse::BadRequest().body("Invalid Path"),
    };

    let pin = req.headers().get("X-User-Pin");
    let pin = match pin {
        Some(pin) => match pin.to_str() {
            Ok(pin) => match pin.parse::<i64>() {
                Ok(pin) => pin,
                Err(_) => return HttpResponse::BadRequest().body("Invalid Pin - no number"),
            },
            Err(_) => return HttpResponse::BadRequest().body("Invalid Pin - no number"),
        },
        None => return HttpResponse::BadRequest().body("No Pin provided in X-User-Pin Header"),
    };

    let res = User::join_game(user_id, to_join, pin, ACTIVE_USERS).await;

    match res {
        Ok(_) => HttpResponse::Ok().body("success".to_string()),
        Err(er) => HttpResponse::InternalServerError().body(er.to_string()),
    }
}

#[patch("/game/{game_id}")]
async fn patch_game(
    path: web::Path<String>,
    body: web::Json<data_source::Game>,
    req: HttpRequest,
) -> impl Responder {
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
        Ok(id) => id,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let body = body.into_inner();

    let res = Game::patch(&_id, GAMES, body).await;

    match res {
        Ok(Some(_)) => HttpResponse::Ok().body("success".to_string()),
        Ok(None) => HttpResponse::NotFound().body("Game not found"),
        Err(er) => HttpResponse::InternalServerError().body(er.to_string()),
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
        Ok(id) => id,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let res = Game::delete(&_id, GAMES).await;

    match res {
        Ok(_) => HttpResponse::Ok().body("success".to_string()),
        Err(er) => HttpResponse::InternalServerError().body(er.to_string()),
    }
}

#[post("/dealer/register")]
async fn register_dealer(body: web::Json<data_source::RegisterDealer>) -> impl Responder {
    let password = body.password.as_str();

    let salt: SaltString = SaltString::generate(&mut OsRng);
    let argon2: Argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password.as_bytes(), &salt);

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
        Ok(_) => HttpResponse::Ok().body("success".to_string()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
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
        None => return HttpResponse::NotFound().body("User not found"),
        Some(u) => u,
    };

    match usr {
        User::Player(_) => HttpResponse::Unauthorized().body("User is not a Dealer".to_string()),
        User::Dealer(u) => {
            let is_aut = u.is_authenticated(password).await;

            match is_aut {
                Ok(o) => {
                    if o {
                        return HttpResponse::Ok().json(json!({
                          "_id": u._id.to_string(),
                        }));
                    }

                    HttpResponse::Unauthorized().body("Wrong Password".to_string())
                }
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
    }
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::PermanentRedirect()
        .insert_header(("Location", "https://www.youtube.com/watch?v=R_ijlnDtKa4"))
        .finish()
}

async fn generate_pins(args: Vec<String>) -> io::Result<()> {
    let location = args.iter().position(|x| x == "-p").unwrap();
    let amount: u64 = args.get(location + 1).unwrap().parse().unwrap();

    let credits: u64 = args.get(location + 2).unwrap().parse().unwrap();

    let mut file = File::create("pins.txt")?;

    for _ in 0..amount {
        let pin: u32 = rand::thread_rng().gen_range(100_000..999_999);

        let data = User::Player(Player {
            name: None,
            nickname: None,
            _id: ObjectId::new(),
            credits: credits,
            pin,
            active_game: None,
        });

        let u = User::new(data, PENDING_USERS).await;

        match u {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e);
                break;
            }
        }

        println!("{}", pin);

        let mut p = pin.to_string();
        p.push('\n');

        file.write_all(p.to_string().as_bytes())?;
    }

    Ok(())
}

async fn create_default_dealer() -> io::Result<()> {
    let password: u64 = OsRng::default().gen();
    let name: u64 = OsRng::default().gen();

    let password = password.to_string();
    let name = name.to_string();

    let salt: SaltString = SaltString::generate(&mut OsRng);
    let argon2: Argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password.as_bytes(), &salt);

    let pw = match password_hash {
        Ok(pass) => pass,
        Err(e) => {
            return Err(io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            ));
        }
    };

    println!("Default dealer created with name: {}", &name);
    println!("Default dealer created with password: {}", &password);

    let d = User::Dealer(Dealer {
        name: name.to_string(),
        _id: ObjectId::new(),
        password: pw.to_string(),
    });

    let r = User::new(d, ACTIVE_USERS).await;

    match r {
        Ok(_) => Ok(()),
        Err(_) => {
            return Err(io::Error::new(
                std::io::ErrorKind::NotFound,
                "Dealer creation failed",
            ));
        }
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"-p".to_string()) {
        generate_pins(args).await?;
    }

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let client = ACTIVE_USERS.get_new_db_client().await?;
    let db = client.database(ACTIVE_USERS.database_identifier);

    let coll: Collection<User> = db.collection(ACTIVE_USERS.collection_identifier);
    let usr_indices = IndexModel::builder().keys(doc! { "name": 1}).build();
    let res = coll
        .create_index(usr_indices)
        .await
        .expect("Cannot create index ACTIVE_USERS");
    info!("Created index: {:?}", res);

    let usr_indices = IndexModel::builder()
        .keys(doc! { "name": 1, "pin": 1, "nickname": 1})
        .build();
    let res = coll
        .create_index(usr_indices)
        .await
        .expect("Cannot create index ACTIVE_USERS");
    info!("Created index: {:?}", res);

    let usr_indices = IndexModel::builder().keys(doc! {"pin": 1}).build();
    let res = coll
        .create_index(usr_indices)
        .await
        .expect("Cannot create index ACTIVE_USERS");
    info!("Created index: {:?}", res);

    let usr_indices = IndexModel::builder().keys(doc! {"role": 1}).build();
    let res = coll
        .create_index(usr_indices)
        .await
        .expect("Cannot create index ACTIVE_USERS");
    info!("Created index: {:?}", res);

    let usr_indices = IndexModel::builder().keys(doc! {"active_game": 1}).build();
    let res = coll
        .create_index(usr_indices)
        .await
        .expect("Cannot create index ACTIVE_USERS");
    info!("Created index: {:?}", res);

    let coll: Collection<User> = db.collection(PENDING_USERS.collection_identifier);
    let pen_usr_indices = IndexModel::builder().keys(doc! { "name": 1}).build();
    let res = coll
        .create_index(pen_usr_indices)
        .await
        .expect("Cannot create index PENDING_USERS");
    info!("Created index: {:?}", res);

    let pen_usr_indices = IndexModel::builder().keys(doc! {"pin": 1}).build();
    let res = coll
        .create_index(pen_usr_indices)
        .await
        .expect("Cannot create index PENDING_USERS");
    info!("Created index: {:?}", res);

    let pen_usr_indices = IndexModel::builder()
        .keys(doc! { "name": 1, "pin": 1})
        .build();
    let res = coll
        .create_index(pen_usr_indices)
        .await
        .expect("Cannot create index PENDING_USERS");
    info!("Created index: {:?}", res);

    let coll: Collection<Game> = db.collection(GAMES.collection_identifier);
    let game_inidces = IndexModel::builder().keys(doc! {"active_game": 1 }).build();
    let res = coll
        .create_index(game_inidces)
        .await
        .expect("Cannot create index GAMES");
    info!("Created index: {:?}", res);

    let coll: Collection<DBUser> = db.collection(ACTIVE_USERS.collection_identifier);
    let res = coll
        .find_one(doc! {"role": "Dealer"})
        .await
        .expect("Cannot find dealer ACTIVE_USERS");

    if res.is_none() {
        let res = create_default_dealer().await;

        match res {
            Ok(_) => {}
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Dealer creation failed",
                ));
            }
        }
    } else {
        println!("Dealer User exist - creating default dealer skipped");
    }

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
            .service(set_user_credits)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
