mod server;
mod my_redis_actor;
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



fn format_redis_result<T>(result: Result<Result<RespValue, T>, MailboxError>) -> String {
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


#[get("/")]
async fn front(redis: web::Data<Addr<RedisActor>>) -> impl Responder {
    let mut body = format!("Hello World!\n");

    let cmd = redis.send(Command(resp_array!["GET", "testkey"]));
    let result = cmd.await;
    body += &format_redis_result(result);
    body += "\n\nclient list = ";
    body += format_redis_result(redis.send(Command(resp_array!["CLIENT", "LIST"])).await).as_str();

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
struct Note {}

#[derive(Serialize)]
struct NoteResponse { ident: String }

#[post("/note/store")]
async fn note_store(note: Json<Note>, redis: web::Data<Addr<RedisActor>>) -> impl Responder {
    HttpResponse::Ok().json(NoteResponse { ident: "".to_string() })
}

#[get("/note/check")]
async fn note_check(redis: web::Data<Addr<RedisActor>>) -> impl Responder {""}

#[post("/note/retrieve")]
async fn note_retrieve(redis: web::Data<Addr<RedisActor>>) -> impl Responder {""}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
    }).bind("127.0.0.1:8080")?.run().await
}