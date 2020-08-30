mod server;
mod my_redis_actor;
mod chatbroker;

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
use chatbroker::ChatMessageBroker;
use crate::chatbroker::{ConnectCmd, DisconnectCmd, ChatMessage, BroadcastCmd};

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
    channel_name: String,
    broker: Addr<ChatMessageBroker>,
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
        self.broker.send(ConnectCmd { addr: ctx.address().recipient(), session: self.channel_name.clone() })
            .into_actor(self)
            .then(|res, _, ctx| {
                if let Err(_) = res {
                    ctx.stop()
                }
                fut::ready(())
            })
            .wait(ctx);

        println!("Websocket connected: {:?}", 0);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.broker.send(DisconnectCmd { addr: ctx.address().recipient(), session: self.channel_name.clone() })
            .into_actor(self)
            .then(|res, _, ctx| {
                if let Err(_) = res {
                    ctx.stop()
                }
                fut::ready(())
            })
            .wait(ctx);
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
                // ctx.text(text)
                self.broker.do_send(BroadcastCmd { session: self.channel_name.clone(), content: text })
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl Handler<ChatMessage> for MyWebSocket {
    type Result = ();

    fn handle(&mut self, msg: ChatMessage, ctx: &mut Self::Context) -> Self::Result {
        println!("receive from broker");
        ctx.text(format!("Test message: {} from {}", msg.content, self.channel_name));
    }
}

impl MyWebSocket {
    fn new(channel_name: String, broker: Addr<ChatMessageBroker>) -> Self {
        Self { hb: Instant::now(), channel_name, broker }
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

#[get("/websocket/{channel}")]
async fn websocket(r: HttpRequest, web::Path(channel): web::Path<String>, stream: web::Payload, broker: web::Data<Addr<ChatMessageBroker>>) -> Result<HttpResponse, weberror::Error> {
    let res = ws::start(MyWebSocket::new(channel, broker.get_ref().clone()), &r, stream);
    res
}


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
            .service(actix_files::Files::new("/static", "/home/markus/Projekte/secretnote/static").show_files_listing())
    }).bind("127.0.0.1:8080")?.run().await
}