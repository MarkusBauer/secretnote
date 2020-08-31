mod chatbroker;
mod chat_websocket;

use std::env;
use actix_web::{get, post, web, App, HttpServer, Responder, middleware, HttpRequest, HttpResponse};
use actix_web::error as weberror;
use actix_files;
use actix_redis::{RedisActor, Command};
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
use actix_files::NamedFile;


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


#[get("/front")]
async fn front(redis: web::Data<Addr<RedisActor>>) -> impl Responder {
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

#[get("/websocket/{channel}")]
async fn websocket(r: HttpRequest, web::Path(channel): web::Path<String>, stream: web::Payload, broker: web::Data<Addr<ChatMessageBroker>>) -> Result<HttpResponse, weberror::Error> {
    ws::start(ChattingWebSocket::new(channel, broker.get_ref().clone()), &r, stream)
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
async fn note_store(note: Json<Note>, redis: web::Data<Addr<RedisActor>>) -> impl Responder {
    let ident = random_string();
    let data = base64::decode(&note.data).unwrap_or(vec![]);
    // validate note text / impose limits
    if data.len() < 16 || data.len() > 1 * 1024 * 1024 {
        return HttpResponse::InternalServerError().body("Message too long or invalid");
    }

    let cmd = Command(resp_array!["SET", format!("note:{}", ident), data, "EX", format!("{}", 3600 * 24 * 7)]);
    //let cmd = Command(resp_array!["SET", format!("note:{}", ident), data]);
    let result = redis.send(cmd).await;
    if let Ok(Ok(RespValue::SimpleString(_))) = result {
        HttpResponse::Ok().json(NoteResponse { ident })
    } else {
        println!("{}", format_redis_result(&result));
        HttpResponse::InternalServerError().body("Redis connection error")
    }
}

#[get("/api/note/check/{ident}")]
async fn note_check(web::Path(ident): web::Path<String>, redis: web::Data<Addr<RedisActor>>) -> impl Responder {
    let data = redis.send(Command(resp_array!["GET", format!("note:{}", ident)])).await;
    if let Ok(Ok(value)) = data {
        match value {
            RespValue::Nil => HttpResponse::Ok().json(CheckNoteResponse { ident: ident, exists: false }),
            RespValue::BulkString(_) => HttpResponse::Ok().json(CheckNoteResponse { ident: ident, exists: true }),
            _ => HttpResponse::InternalServerError().body("Invalid data type in redis")
        }
    } else {
        println!("{}", format_redis_result(&data));
        HttpResponse::InternalServerError().body("Redis connection error")
    }
}

#[post("/api/note/retrieve")]
async fn note_retrieve(note: Json<RetrieveNoteRequest>, redis: web::Data<Addr<RedisActor>>) -> impl Responder {
    let data = redis.send(Command(resp_array!["GET", format!("note:{}", &note.ident)])).await;
    if let Ok(Ok(value)) = data {
        if let RespValue::BulkString(vec) = value {
            redis.do_send(Command(resp_array!["DEL", format!("note:{}", &note.ident)]));
            HttpResponse::Ok().json(RetrieveNoteResponse { ident: note.ident.clone(), data: base64::encode(vec) })
        } else {
            HttpResponse::InternalServerError().body("Invalid data type in redis")
        }
    } else {
        println!("{}", format_redis_result(&data));
        HttpResponse::InternalServerError().body("Redis connection error")
    }
}


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

async fn angular_index() -> std::io::Result<NamedFile> {
    NamedFile::open(get_base_path().join("fe").join("index.html").clone())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let basepath = get_base_path();
    println!("base path = \"{}\"", basepath.to_str().unwrap());

    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let broker = ChatMessageBroker::default().start();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(RedisActor::start("127.0.0.1:6379"))
            //.data(RedisPubsubActorV2::start("127.0.0.1:6379"))
            .data(broker.clone())
            .service(front)
            .service(index)
            .service(websocket)
            .service(note_store).service(note_check).service(note_retrieve)
            .service(actix_files::Files::new("/static", "/home/markus/Projekte/secretnote/static").show_files_listing())
            .service(web::resource("/note/*").to(angular_index))
            .service(web::resource("/faq").to(angular_index))
            .service(actix_files::Files::new("/", basepath.join("fe")).index_file("index.html"))
    }).bind("127.0.0.1:8080")?.run().await
}