use std::env;
use actix_web::{get, web, App, HttpServer, Responder, middleware, HttpRequest, HttpResponse};
use actix_web::error as weberror;
use actix_files;
use actix_redis::{RedisActor, Command};
use actix::prelude::*;
use actix_web_actors::ws;
use redis_async::resp_array;
use redis_async::resp::RespValue;
use std::time::{Duration, Instant};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


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


/// websocket connection is long running connection, it easier
/// to handle with an actor
struct MyWebSocket {
    hb: Instant,
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
        println!("Websocket connected: {:?}", 0);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Websocket disconnected: {:?}", 0);
    }
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                ctx.text(text)
            },
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl MyWebSocket {
    fn new() -> Self {
        Self { hb: Instant::now() }
    }

    /// helper method that sends ping to client every second.
    /// also this method checks heartbeats from client
    fn heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                ctx.stop();
                return;
            }
            // otherwise ping
            ctx.ping(b"");
        });
    }
}

#[get("/websocket")]
async fn websocket(r: HttpRequest, stream: web::Payload, redis: web::Data<Addr<RedisActor>>) -> Result<HttpResponse, weberror::Error> {
    let res = ws::start(MyWebSocket::new(), &r, stream);
    res
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .data(RedisActor::start("127.0.0.1:6379"))
            .service(front)
            .service(index)
            .service(websocket)
            .service(actix_files::Files::new("/static", "/home/markus/Projekte/secretnote/static").show_files_listing())
    }).bind("127.0.0.1:8080")?.run().await
}