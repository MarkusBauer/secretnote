mod chatbroker;
mod chat_websocket;
mod my_redis_actor;

use std::{env, fs};
use actix_web::{get, post, web, App, HttpServer, Responder, middleware, HttpRequest, HttpResponse};
use actix_web::error as weberror;
use actix_files;
use actix_redis::{Command};
use actix::prelude::*;
use actix_web_actors::ws;
use redis_async::resp_array;
use redis_async::resp::RespValue;
use crate::chatbroker::{ChatMessageBroker};
use crate::chat_websocket::{ChattingWebSocket};
use actix_web::web::Json;
use serde::{Deserialize, Serialize};
use rand::{Rng, thread_rng};
use base64;
use rand::distributions::Alphanumeric;
use std::path::PathBuf;
use cached::proc_macro::cached;
use std::sync::Arc;
use crate::my_redis_actor::MyRedisActor;


fn format_redis_result<T>(result: &Result<Result<RespValue, T>, MailboxError>) -> String {
    match result {
        Err(_) => format!("<error 1>"),
        Ok(Err(_)) => format!("<error 2>"),
        Ok(Ok(x)) => {
            match x {
                RespValue::Nil => format!("nil"),
                RespValue::Array(y) => format!("{} array elements", y.len()),
                RespValue::BulkString(bytes) => format!("{} bytes: \"{}\"", bytes.len(), std::str::from_utf8(&bytes).unwrap()),
                RespValue::Error(e) => format!("{:#}", e),
                RespValue::Integer(i) => format!("int {}", i),
                RespValue::SimpleString(s) => format!("\"{}\"", s),
            }
        }
    }
}

fn random_string() -> String {
    return thread_rng().sample_iter(&Alphanumeric).take(24).collect();
}


/*
#[get("/front")]
async fn front(redis: web::Data<Addr<MyRedisActor>>) -> impl Responder {
    let mut body = format!("Hello World!\n");

    let cmd = redis.send(Command(resp_array!["GET", "testkey"]));
    let result = cmd.await;
    body += &format_redis_result(&result);
    body += "\n\nclient list = ";
    body += format_redis_result(&redis.send(Command(resp_array!["CLIENT", "LIST"])).await).as_str();

    body
}

#[get("/{id}/{name}/index.html")]
async fn index(web::Path((id, name)): web::Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", name, id)
}
 */


#[derive(Deserialize)]
struct ChatMessagesRequest { offset: isize, total_count: isize, limit: isize }

#[derive(Serialize)]
struct ChatMessageResponse { len: i64, messages: Vec<String> }

#[get("/api/chat/websocket/{channel}")]
async fn websocket(r: HttpRequest,
                   web::Path(channel): web::Path<String>,
                   stream: web::Payload,
                   broker: web::Data<Addr<ChatMessageBroker>>,
                   redis: web::Data<Addr<MyRedisActor>>) -> Result<HttpResponse, weberror::Error> {
    ws::start(ChattingWebSocket::new(channel, broker.get_ref().clone(), redis.get_ref().clone()), &r, stream)
}

#[post("/api/chat/messages/{channel}")]
async fn chat_messages(web::Path(channel): web::Path<String>, body: Json<ChatMessagesRequest>, redis: web::Data<Addr<MyRedisActor>>) -> impl Responder {
    let start = body.offset - body.total_count;
    let stop = start + body.limit - 1;
    let stop = if start < 0 && stop >= 0 { -1 } else { stop };
    let result = redis.send(Command(resp_array!["LLEN", format!("chat:{}", channel)])).await;
    if let Ok(Ok(RespValue::Integer(len))) = result {
        // println!("len={}  start={}  stop={}", len, start, stop);
        let result = redis.send(Command(resp_array!["LRANGE", format!("chat:{}", channel), format!("{}", start), format!("{}", stop)])).await;
        if let Ok(Ok(RespValue::Array(responses))) = result {
            let mut response = ChatMessageResponse { len, messages: vec![] };
            for r in responses {
                if let RespValue::BulkString(bin) = r {
                    response.messages.push(base64::encode(bin));
                } else {
                    println!("Redis returned response that was not a binary string");
                }
            }
            HttpResponse::Ok().header("Cache-Control", "no-cache, no-store").json(response)
        } else {
            HttpResponse::InternalServerError().header("Cache-Control", "no-cache, no-store").body("LRANGE call failed")
        }
    } else {
        HttpResponse::InternalServerError().header("Cache-Control", "no-cache, no-store").body("LLEN call failed")
    }
}


#[derive(Deserialize)]
struct Note {
    /// base64-encoded crypted data
    data: String
}

#[derive(Serialize)]
struct NoteResponse { ident: String }

#[derive(Serialize)]
struct CheckNoteResponse { ident: String, exists: bool }

#[derive(Deserialize)]
struct RetrieveNoteRequest { ident: String }

#[derive(Serialize)]
struct RetrieveNoteResponse { ident: String, data: String }

#[post("/api/note/store")]
async fn note_store(note: Json<Note>, redis: web::Data<Addr<MyRedisActor>>) -> impl Responder {
    let ident = random_string();
    let data = base64::decode(&note.data).unwrap_or(vec![]);
    // validate note text / impose limits
    if data.len() < 16 || data.len() > 1 * 1024 * 1024 {
        return HttpResponse::InternalServerError().header("Cache-Control", "no-cache, no-store").body("Message too long or invalid");
    }

    let cmd = Command(resp_array!["SET", format!("note:{}", ident), data, "EX", format!("{}", 3600 * 24 * 7)]);
    //let cmd = Command(resp_array!["SET", format!("note:{}", ident), data]);
    let result = redis.send(cmd).await;
    if let Ok(Ok(RespValue::SimpleString(_))) = result {
        HttpResponse::Ok().header("Cache-Control", "no-cache, no-store")
            .json(NoteResponse { ident })
    } else {
        println!("{}", format_redis_result(&result));
        HttpResponse::InternalServerError().header("Cache-Control", "no-cache, no-store").body("Redis connection error")
    }
}

#[get("/api/note/check/{ident}")]
async fn note_check(web::Path(ident): web::Path<String>, redis: web::Data<Addr<MyRedisActor>>) -> impl Responder {
    let data = redis.send(Command(resp_array!["GET", format!("note:{}", ident)])).await;
    if let Ok(Ok(value)) = data {
        match value {
            RespValue::Nil => HttpResponse::Ok().json(CheckNoteResponse { ident: ident, exists: false }),
            RespValue::BulkString(_) => HttpResponse::Ok().header("Cache-Control", "no-cache, no-store")
                .json(CheckNoteResponse { ident: ident, exists: true }),
            _ => HttpResponse::InternalServerError().header("Cache-Control", "no-cache, no-store").body("Invalid data type in redis")
        }
    } else {
        println!("{}", format_redis_result(&data));
        HttpResponse::InternalServerError().header("Cache-Control", "no-cache, no-store").body("Redis connection error")
    }
}

#[post("/api/note/retrieve")]
async fn note_retrieve(note: Json<RetrieveNoteRequest>, redis: web::Data<Addr<MyRedisActor>>) -> impl Responder {
    let data = redis.send(Command(resp_array!["GET", format!("note:{}", &note.ident)])).await;
    if let Ok(Ok(value)) = data {
        if let RespValue::BulkString(vec) = value {
            redis.do_send(Command(resp_array!["DEL", format!("note:{}", &note.ident)]));
            HttpResponse::Ok().header("Cache-Control", "no-cache, no-store")
                .json(RetrieveNoteResponse { ident: note.ident.clone(), data: base64::encode(vec) })
        } else {
            HttpResponse::InternalServerError().header("Cache-Control", "no-cache, no-store").body("Invalid data type in redis")
        }
    } else {
        println!("{}", format_redis_result(&data));
        HttpResponse::InternalServerError().header("Cache-Control", "no-cache, no-store").body("Redis connection error")
    }
}


#[cached]
fn get_base_path() -> PathBuf {
    let pathbuf = std::env::current_exe().unwrap().clone();
    let pathbuf2 = pathbuf.canonicalize().unwrap();
    let base = pathbuf2.parent().unwrap().to_path_buf();
    if base.file_name().unwrap() == "debug" && base.parent().unwrap().file_name().unwrap() == "target" {
        base.parent().unwrap().parent().unwrap().to_path_buf()
    } else {
        base
    }
}

#[cached]
fn read_index() -> String {
    fs::read_to_string(get_base_path().join("fe").join("index.html")).unwrap()
}

async fn angular_index() -> impl Responder {
    // let f = NamedFile::open(get_base_path().join("fe").join("index.html").clone());
    HttpResponse::Ok()
        .content_type("text/html; charset=UTF-8")
        .header("Cache-Control", "must-revalidate, max-age=3600")
        .header("Content-Security-Policy", "default-src 'self'; style-src 'self' 'unsafe-inline'; base-uri 'self'; form-action 'self'; frame-ancestors 'self'; object-src 'none';")
        .header("X-Content-Type-Options", "nosniff")
        .header("X-Frame-Options", "SAMEORIGIN")
        .header("Referrer-Policy", "no-referrer")
        .body(read_index())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = clap::App::new("SecretNote Server")
        //.version("1.0")
        //.author("...")
        .about("Hosts SecretNote - a cryptographic note and chat platform")
        .arg(clap::Arg::with_name("bind")
            .short('b')
            .long("bind")
            .value_name("BIND")
            .about("Sets which ip/port should be bound")
            .takes_value(true))
        .arg(clap::Arg::with_name("redis")
            .long("redis")
            .value_name("HOST:PORT")
            .about("Sets which ip/port should be bound")
            .takes_value(true))
        .arg(clap::Arg::with_name("redis-db")
            .long("redis-db")
            .value_name("DB")
            .about("database number")
            .takes_value(true))
        .arg(clap::Arg::with_name("redis-auth")
            .long("redis-auth")
            .value_name("PASSWORD")
            .about("Sets a password for the redis database")
            .takes_value(true))
        .arg(clap::Arg::with_name("verbose")
            .long("verbose")
            .short('v')
            .about("Set verbose mode (enable request logs)"))
        .get_matches();

    let basepath = get_base_path();
    println!("Frontend at \"{}/fe\"", basepath.to_str().unwrap());

    if matches.is_present("verbose") {
        env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
        env_logger::init();
    }
    let redis: Arc<String> = Arc::new(matches.value_of("redis").unwrap_or("127.0.0.1:6379").into());
    let redis_db: u32 = matches.value_of_t("redis-db").unwrap_or(0);
    let redis_auth = Arc::new(if let Some(x) = matches.value_of("redis-auth") { Some(String::from(x)) } else { None });
    let bind: String = matches.value_of("bind").unwrap_or("127.0.0.1:8080").into();
    println!("Using Redis at \"{}\" ...", redis);
    println!("Binding to \"{}\" ...", bind);

    let broker = ChatMessageBroker::default().start();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::DefaultHeaders::new().header("Cache-Control", "max-age=2592000"))
            .data(MyRedisActor::start((*redis).clone(), Some(redis_db), (*redis_auth).clone()))
            //.data(RedisPubsubActorV2::start("127.0.0.1:6379"))
            .data(broker.clone())
            //.service(front)
            //.service(index)
            .service(websocket)
            .service(note_store)
            .service(note_check)
            .service(note_retrieve)
            .service(chat_messages)
            // .service(actix_files::Files::new("/static", "/home/markus/Projekte/secretnote/static").show_files_listing())
            .service(web::resource("/note/*").to(angular_index))
            .service(web::resource("/chat/*").to(angular_index))
            .service(web::resource("/faq").to(angular_index))
            .service(web::resource("/").to(angular_index))
            .service(actix_files::Files::new("/", basepath.join("fe")).use_last_modified(true).use_etag(true))
    }).bind(bind)?.run().await
}