use actix_redis::Command;
use actix_web::{HttpResponse, web, Responder};
use serde_json::value::Value::Object;
use bytes::Bytes;
use crate::my_redis_actor::MyRedisActor;
use actix::{Addr, Context, Actor, Handler, Message, WrapFuture, ContextFutureSpawner, ActorFuture, AsyncContext};
use serde::{Serialize, Deserialize};
use redis_async::resp_array;
use redis_async::resp::RespValue;
use actix_web::client::{Client, ClientResponse, SendRequestError, JsonPayloadError};
use serde_json::Value;
use futures::{Future};
use actix_http::{Payload, PayloadStream};
use actix_http::encoding::Decoder;
use serde_json::json;


pub async fn send_read_confirmation(config: &str, ident: &str, redis: &Addr<MyRedisActor>, telegram: &Addr<TelegramActor>) {
    if config.starts_with("telegram:") {
        let chat_id = match config[9..].parse::<i64>() {
            Ok(x) => x,
            _ => {
                if config[9..].starts_with("@") {
                    let result = redis.send(Command(resp_array!["GET", format!("telegram.user_to_chat:{}", &config[10..])])).await;
                    if let Ok(Ok(RespValue::BulkString(vec))) = result {
                        std::str::from_utf8(&vec).unwrap_or("").parse::<i64>().unwrap_or(0)
                    } else { 0 }
                } else { 0 }
            }
        };
        telegram.do_send(SendMessage{chat_id, text: format!("Activity at SecretNote: Your message with ID _{}_ has just been read\\.", ident), parse_mode: "MarkdownV2".into()});
    }
}

pub fn store_telegram_read_notification(ident: &str, chat: &str, redis: &Addr<MyRedisActor>) {
    redis.do_send(Command(resp_array!["SET", format!("note_settings:read_confirmation:{}", ident), format!("telegram:{}", chat)]));
}

fn escape_markdown(s: &str) -> String {
    let mut s: String = s.into();
    s = s.replace("\\", "\\\\");
    s = s.replace("_", "\\_");
    s = s.replace("*", "\\*");
    s = s.replace("[", "\\[");
    s = s.replace("]", "\\]");
    s = s.replace("(", "\\(");
    s = s.replace(")", "\\)");
    s = s.replace("~", "\\~");
    s = s.replace("`", "\\`");
    s = s.replace(">", "\\>");
    s = s.replace("#", "\\#");
    s = s.replace("+", "\\+");
    s = s.replace("-", "\\-");
    s = s.replace("=", "\\=");
    s = s.replace("|", "\\|");
    s = s.replace("{", "\\{");
    s = s.replace("}", "\\}");
    s = s.replace(".", "\\.");
    s = s.replace("!", "\\!");
    return s;
}


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
                               Please use your *chat ID \"{}\"* or your *username \"@{}\"* after storing a message.\n\
                               You can also send your admin links to this bot.",
                              chat_id, escape_markdown(username.unwrap_or("???"))
            );
            return HttpResponse::Ok().json(TelegramWebhookMessageResponse {
                method: "sendMessage".into(),
                chat_id,
                text: msg,
                parse_mode: "MarkdownV2".into(),
            });
        }
        // TODO parse other messages
        telegram.do_send(SendMessage {
            chat_id,
            text: "Sorry, bot could not understand your message.".to_string(),
            parse_mode: "".to_string(),
        });//*/
    }
    HttpResponse::Ok().body("ok")
}


#[derive(Debug, Serialize, Deserialize)]
pub struct TelegramApiResult {
    ok: bool,
    result: Option<Value>,
    description: Option<String>,
}


#[derive(Default)]
pub struct TelegramActor {
    pub token: String,
    pub webhook_url: String,
    pub available: bool,
    pub username: String,
}

impl TelegramActor {
    pub fn new(token: &str, webhook_url: &str) -> TelegramActor {
        return TelegramActor { token: token.into(), webhook_url: webhook_url.into(), available: false, username: "".into() };
    }

    pub fn get_url(&self, method: &str) -> String {
        let mut s: String = "https://api.telegram.org/bot".into();
        s.push_str(&self.token);
        s.push_str("/");
        s.push_str(method);
        return s;
    }

    pub fn send_api_request<T: 'static + Future<Output=Result<ClientResponse<Decoder<Payload<PayloadStream>>>, SendRequestError>>, F: 'static>(
        &self,
        request: T,
        ctx: &mut <TelegramActor as Actor>::Context,
        callback: F,
    ) where F: FnOnce(TelegramApiResult, &mut TelegramActor, &mut <TelegramActor as Actor>::Context) -> () {
        request.into_actor(self)
            .map(|res, act, ctx| {
                match res {
                    Ok(mut result) => {
                        result.json().into_actor(act).map(|j: Result<TelegramApiResult, JsonPayloadError>, act, ctx| {
                            match j {
                                Ok(j) => { callback(j, act, ctx); }
                                Err(err) => { println!("Telegram: Error in json response: {:?}", err); }
                            }
                        }).wait(ctx);
                    }
                    Err(err) => {
                        println!("Telegram API error: {:?}", err);
                    }
                }
            })
            .wait(ctx);
    }
}

impl Actor for TelegramActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        if self.token.is_empty() {
            return;
        }
        // Initialize twice - sometimes first try fails.
        ctx.address().do_send(Initialize {});
        ctx.address().do_send(Initialize {});
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
        Client::default().post(&url).content_type("application/json; charset=utf-8").send_json(&msg)
            .into_actor(self)
            .map(|res, _act, _ctx| {
                if let Err(err) = res {
                    println!("Telegram API error: /sendMessage {:?}", err);
                }
            })
            .wait(ctx);
    }
}

pub struct Initialize {}

impl Message for Initialize {
    type Result = ();
}

impl Handler<Initialize> for TelegramActor {
    type Result = ();

    fn handle(&mut self, _msg: Initialize, ctx: &mut Context<Self>) {
        if self.available {
            return;
        }
        println!("Telegram Bot: Trying to initialize bot ...");
        let req = Client::default().post(self.get_url("getMe")).send();
        self.send_api_request(req, ctx, |response, act, ctx| {
            println!("Response: {:?}", response);
            if !response.ok {
                println!("Telegram Bot: Request /getMe failed! Bot is disabled.");
                return;
            }
            let username: String = response.result.unwrap_or(Value::Null).get("username").unwrap_or(&Value::Null).as_str().unwrap_or("").into();
            if username.is_empty() {
                println!("Telegram Bot: No username present, bot is disabled.");
                return;
            }
            act.available = true;
            act.username = username.into();
            ctx.address().do_send(CheckWebhook {});
        });
    }
}


pub struct CheckWebhook {}

impl Message for CheckWebhook {
    type Result = ();
}

impl Handler<CheckWebhook> for TelegramActor {
    type Result = ();

    fn handle(&mut self, _cmd: CheckWebhook, ctx: &mut Context<Self>) {
        // check/set webhook
        if self.webhook_url.is_empty() { return; }
        self.send_api_request(Client::default().get(self.get_url("getWebhookInfo")).send(), ctx, |response, act, ctx| {
            if !response.ok { return; }
            let current_url: String = response.result.unwrap_or(Value::Null).get("url").unwrap_or(&Value::Null).as_str().unwrap_or("").into();
            println!("Telegram Bot: Current webhook: \"{}\", new webhook: \"{}\"", current_url, act.webhook_url);
            if current_url != act.webhook_url {
                // Set webhook to current url
                ctx.address().do_send(SetWebhook { webhook_url: act.webhook_url.clone() });
            }
        });
    }
}


pub struct SetWebhook {
    pub webhook_url: String
}

impl Message for SetWebhook {
    type Result = ();
}

impl Handler<SetWebhook> for TelegramActor {
    type Result = ();

    fn handle(&mut self, cmd: SetWebhook, ctx: &mut Context<Self>) {
        let payload = json!({"url": cmd.webhook_url});
        let req = Client::default().post(self.get_url("setWebhook")).send_json(&payload);
        self.send_api_request(req, ctx, |response, _act, _ctx| {
            if response.ok {
                println!("Telegram Bot: Webhook has been changed.");
            }
        });
    }
}
