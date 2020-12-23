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
use crypto::sha2::Sha256;
use crypto::digest::Digest;
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::Value;
use serde_json::value::Value::Object;
use bytes::Bytes;


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

fn hash_ident(admin_ident: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.input_str(admin_ident);
    let mut v: Vec<u8> = Vec::with_capacity(32);
    v.resize(32, 0u8);
    hasher.result(&mut v);
    let s = base64::encode_config(&v, base64::URL_SAFE_NO_PAD);
    println!("admin_ident = \"{}\"   ident = \"{}\"", &admin_ident, &s[..28]);
    String::from(&s[..28])
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
struct NoteResponse { ident: String, admin_ident: String }

#[derive(Serialize)]
struct CheckNoteResponse { ident: String, exists: bool }

#[derive(Deserialize)]
struct RetrieveNoteRequest { ident: String }

#[derive(Serialize)]
struct RetrieveNoteResponse { ident: String, data: String }

#[post("/api/note/store")]
async fn note_store(note: Json<Note>, redis: web::Data<Addr<MyRedisActor>>) -> impl Responder {
    let admin_ident = random_string();
    let ident = hash_ident(&admin_ident);
    let data = base64::decode(&note.data).unwrap_or(vec![]);
    // validate note text / impose limits
    if data.len() < 16 || data.len() > 1 * 1024 * 1024 {
        return HttpResponse::InternalServerError().header("Cache-Control", "no-cache, no-store").body("Message too long or invalid");
    }

    let bytes = data.len();
    let cmd = Command(resp_array!["SET", format!("note:{}", ident), data, "EX", format!("{}", 3600 * 24 * 7)]);
    let result = redis.send(cmd).await;
    if let Ok(Ok(RespValue::SimpleString(_))) = result {
        let cmd = Command(resp_array!["SET", format!("noteadmin:{}", admin_ident), "{}", "EX", format!("{}", 3600 * 24 * 7)]);
        let result = redis.send(cmd).await;
        if let Ok(Ok(RespValue::SimpleString(_))) = result {
            redis.do_send(Command(resp_array!["INCR", "secretnote-stats:note-store-count"]));
            redis.do_send(Command(resp_array!["INCRBY", "secretnote-stats:note-store-bytes", format!("{}", bytes)]));
            HttpResponse::Ok().header("Cache-Control", "no-cache, no-store")
                .json(NoteResponse { ident, admin_ident })
        } else {
            println!("{}", format_redis_result(&result));
            HttpResponse::InternalServerError().header("Cache-Control", "no-cache, no-store").body("Redis connection error")
        }
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
            redis.do_send(Command(resp_array!["INCR", "secretnote-stats:note-retrieve-count"]));
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


#[derive(Serialize)]
struct TelegramWebhookMessageResponse {
    method: String, chat_id: i64, text: String, parse_mode: String
}


async fn telegram_message(body: Bytes, redis: web::Data<Addr<MyRedisActor>>) -> impl Responder {
    let request = match serde_json::from_slice(body.as_ref()) {
        Ok(Object(ok)) => ok,
        _ => {
            return HttpResponse::InternalServerError().body("Invalid JSON");
        }
    };
    if let Some(message) = request.get("message") {
        // handle incoming telegram messages
        let chat_id = message.get("chat").and_then(|chat| chat.get("id")).and_then(|id| id.as_i64()).unwrap_or(0);
        let username = message.get("chat").and_then(|chat| chat.get("username")).and_then(|username| username.as_str());
        if let Some(username) = username {
            println!("Received message from @{} in chat {}", username, chat_id);
            redis.do_send(Command(resp_array!["SET", format!("telegram.user_to_chat:{}", username), format!("{}", chat_id)]));
        }

        let text = message.get("text").and_then(|txt| txt.as_str()).unwrap_or("");
        if text == "/start" {
            let msg = format!("Welcome to SecretNoteBot - you can now receive read notifications for your messages!\n\
                               Please use your *chat ID {}* or your *username* \"@{}\" after storing a message.\n\
                               You can also send your admin links to this bot.",
                chat_id, username.unwrap_or("???")
            );
            return HttpResponse::Ok().json(TelegramWebhookMessageResponse{
                method: "sendMessage".into(), chat_id, text: msg, parse_mode: "MarkdownV2".into()
            });
        }
        // TODO parse other messages
    }
    HttpResponse::Ok().body("ok")
}


#[cached]
fn get_base_path() -> PathBuf {
    let pathbuf = std::env::current_exe().unwrap().clone();
    let pathbuf2 = pathbuf.canonicalize().unwrap();
    let base = pathbuf2.parent().unwrap().to_path_buf();
    if base.file_name().is_some() && (base.file_name().unwrap() == "debug" || base.file_name().unwrap() == "release") && base.parent().unwrap().file_name().unwrap() == "target" {
        base.parent().unwrap().parent().unwrap().to_path_buf()
    } else {
        base
    }
}

#[cached]
fn read_index(language: &'static str, path: String) -> (String, SystemTime) {
    let mut filepath = get_base_path().join("fe").join(language);

    // Check if a matching path exists
    // Folder structure is like: URL "/about" => FILE "/about/index.html"
    let candidate_path = filepath.join(&path[1..]).join("index.html");
    let meta = fs::metadata(&candidate_path);
    if let Ok(meta) = meta {
        return (fs::read_to_string(candidate_path).unwrap(), meta.modified().unwrap());
    }

    // Fallback - this file works with every page.
    // It contains only the AppComponent (e.g. main menu), router-outlet is empty.
    filepath = filepath.join("index.all.html");
    let meta = fs::metadata(&filepath).unwrap();
    (fs::read_to_string(filepath).unwrap(), meta.modified().unwrap())
}

async fn angular_index(req: &HttpRequest, language: &'static str) -> impl Responder {
    // Select which file to use
    let mut path = req.uri().path();
    if path.starts_with(&format!("/{}/", language)) {
        path = &path[language.len() + 1..];
    }
    let (content, mtime) = read_index(language, path.into());
    let epoch = mtime.duration_since(UNIX_EPOCH).unwrap().as_millis();
    let etag = format!("\"{}S{}\"", epoch, content.len());

    let mut response = HttpResponse::Ok();
    let mut need_content = true;
    if let Some(ifmatch_hdr) = req.headers().get("If-None-Match") {
        if let Ok(ifmatch) = ifmatch_hdr.to_str() {
            if ifmatch == etag {
                response = HttpResponse::NotModified();
                need_content = false;
            }
        }
    }
    response
        .header("Cache-Control", "must-revalidate, max-age=3600")
        //.header("Cache-Control", "no-cache")
        .header("ETag", etag)
        .header("Content-Security-Policy", "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; base-uri 'self'; form-action 'self'; frame-ancestors 'self'; object-src 'none';")
        .header("X-Content-Type-Options", "nosniff")
        .header("X-Frame-Options", "SAMEORIGIN")
        .header("Referrer-Policy", "no-referrer");

    if need_content {
        response.content_type("text/html; charset=UTF-8")
            .body(content)
    } else {
        response.body("")
    }
}

async fn angular_index_de(req: HttpRequest) -> impl Responder {
    angular_index(&req, "de").await
}

async fn angular_index_en(req: HttpRequest) -> impl Responder {
    angular_index(&req, "en").await
}

async fn angular_index_any(req: HttpRequest) -> impl Responder {
    if let Some(header) = req.headers().get("Accept-Language") {
        if let Some(lang) = header.to_str().ok() {
            let en = lang.find("en").unwrap_or(usize::max_value() - 1);
            let de = lang.find("de").unwrap_or(usize::max_value());
            if de < en {
                return angular_index(&req, "de").await;
            }
        }
    }
    angular_index(&req, "en").await
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = clap::App::new("SecretNote Server")
        .version("1.0.0")
        //.author("...")
        .about("Hosts SecretNote - a cryptographic note and chat platform")
        .arg(clap::Arg::new("bind")
            .short('b')
            .long("bind")
            .value_name("BIND")
            .about("Sets which ip/port should be bound")
            .takes_value(true))
        .arg(clap::Arg::new("redis")
            .long("redis")
            .value_name("HOST:PORT")
            .about("Sets which ip/port should be bound")
            .takes_value(true))
        .arg(clap::Arg::new("redis-db")
            .long("redis-db")
            .value_name("DB")
            .about("database number")
            .takes_value(true))
        .arg(clap::Arg::new("redis-auth")
            .long("redis-auth")
            .value_name("PASSWORD")
            .about("Sets a password for the redis database")
            .takes_value(true))
        .arg(clap::Arg::new("threads")
            .long("threads")
            .short('t')
            .value_name("NUM_THREADS")
            .about("Number of worker threads to use")
            .takes_value(true))
        .arg(clap::Arg::new("verbose")
            .long("verbose")
            .short('v')
            .about("Set verbose mode (enable request logs)"))
        .arg(clap::Arg::new("telegram-token")
            .long("telegram-token")
            .about("Set the API token of the telegram bot")
            .value_name("TELEGRAM_TOKEN")
            .takes_value(true))
        .get_matches();

    let basepath = get_base_path();
    println!("Frontend at \"{}\"", basepath.join("fe").to_str().unwrap());

    if matches.is_present("verbose") || env::var("SECRETNOTE_VERBOSE").is_ok() {
        env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
        env_logger::init();
    }
    let redis: Arc<String> = Arc::new(matches.value_of("redis").unwrap_or(&env::var("SECRETNOTE_REDIS").unwrap_or("127.0.0.1:6379".into())).into());
    let redis_db: u32 = matches.value_of_t("redis-db").unwrap_or(env::var("SECRETNOTE_REDIS_DB").unwrap_or("0".into()).parse().expect("Redis database must be a number"));
    let redis_auth = Arc::new(if let Some(x) = matches.value_of("redis-auth") { Some(String::from(x)) } else { env::var("SECRETNOTE_REDIS_AUTH").ok() });
    let bind: String = matches.value_of("bind").unwrap_or(&env::var("SECRETNOTE_BIND").unwrap_or("127.0.0.1:8080".into())).into();
    let threads: usize = matches.value_of_t("threads").unwrap_or(env::var("SECRETNOTE_THREADS").unwrap_or("".into()).parse().unwrap_or(num_cpus::get()));
    let telegram_token: Arc<String> = Arc::new(matches.value_of("telegram-token").unwrap_or(&env::var("SECRETNOTE_TELEGRAM_TOKEN").unwrap_or("".into())).into());
    println!("Using Redis at \"{}\", database {} ...", redis, redis_db);
    println!("Binding to \"{}\" ...", bind);
    println!("Starting {} threads ...", threads);

    let broker = ChatMessageBroker::default().start();

    HttpServer::new(move || {
        let mut app = App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::DefaultHeaders::new().header("Cache-Control", "max-age=5184000"))
            .data(MyRedisActor::start((*redis).clone(), Some(redis_db), (*redis_auth).clone()))
            .data(broker.clone())
            .service(websocket)
            .service(note_store)
            .service(note_check)
            .service(note_retrieve)
            .service(chat_messages);

        if !telegram_token.is_empty() {
            let mut s: String = "/api/telegram/webhook/".to_owned();
            s.push_str(telegram_token.as_str());
            app = app.service(web::resource(s).route(web::post().to(telegram_message)));
        }

        app = app
            .service(web::resource("/note/*").to(angular_index_any))
            .service(web::resource("/chat/*").to(angular_index_any))
            .service(web::resource("/faq").to(angular_index_any))
            .service(web::resource("/about").to(angular_index_any))
            .service(web::resource("/").to(angular_index_any))

            .service(web::resource("/de/note/*").to(angular_index_de))
            .service(web::resource("/de/chat/*").to(angular_index_de))
            .service(web::resource("/de/faq").to(angular_index_de))
            .service(web::resource("/de/about").to(angular_index_de))
            .service(web::resource("/de/").to(angular_index_de))

            .service(web::resource("/en/note/*").to(angular_index_en))
            .service(web::resource("/en/chat/*").to(angular_index_en))
            .service(web::resource("/en/faq").to(angular_index_en))
            .service(web::resource("/en/about").to(angular_index_en))
            .service(web::resource("/en/").to(angular_index_en))

            .service(actix_files::Files::new("/", basepath.join("fe")).use_last_modified(true).use_etag(true));
        app
    }).workers(threads).bind(bind)?.run().await
}