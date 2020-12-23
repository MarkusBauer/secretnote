use actix_redis::Command;
use actix_web::{HttpResponse, web, Responder};
use serde_json::value::Value::Object;
use bytes::Bytes;
use crate::my_redis_actor::MyRedisActor;
use actix::{Addr, Context, Actor, Handler, Message, WrapFuture, ContextFutureSpawner, ActorFuture};
use serde::{Serialize};
use redis_async::resp_array;
use actix_web::client::Client;


#[derive(Serialize)]
struct TelegramWebhookMessageResponse {
    method: String,
    chat_id: i64,
    text: String,
    parse_mode: String,
}


pub async fn telegram_message(body: Bytes, redis: web::Data<Addr<MyRedisActor>>, telegram: web::Data<Addr<TelegramActor>>) -> impl Responder {
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
            return HttpResponse::Ok().json(TelegramWebhookMessageResponse {
                method: "sendMessage".into(),
                chat_id,
                text: msg,
                parse_mode: "MarkdownV2".into(),
            });
        }
        // TODO parse other messages
        /*telegram.do_send(SendMessage{
            chat_id,
            text: "Hello World 3".to_string(),
            parse_mode: "MarkdownV2".to_string()
        });*/
    }
    HttpResponse::Ok().body("ok")
}


#[derive(Default)]
pub struct TelegramActor {
    pub token: String,
}

impl TelegramActor {
    pub fn new(token: &str) -> TelegramActor {
        return TelegramActor { token: token.into() };
    }

    pub fn get_url(&self, method: &str) -> String {
        let mut s: String = "https://api.telegram.org/bot".into();
        s.push_str(&self.token);
        s.push_str("/");
        s.push_str(method);
        return s;
    }
}

impl Actor for TelegramActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        // TODO
    }
}


#[derive(Debug)]
#[derive(Serialize)]
pub struct SendMessage {
    pub chat_id: i64,
    pub text: String,
    pub parse_mode: String,
}

impl Message for SendMessage {
    type Result = ();
}


impl Handler<SendMessage> for TelegramActor {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, ctx: &mut Context<Self>) {
        // Send a Telegram message
        let url = self.get_url("sendMessage");
        Client::default().post(&url).send_json(&msg)
            .into_actor(self)
            .map(|res, _act, _ctx| {
                if let Err(err) = res {
                    println!("Telegram API error: /sendMessage {:?}", err);
                }
            })
            .wait(ctx);
    }
}