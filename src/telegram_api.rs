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
use regex::Regex;
use lazy_static::{lazy_static};
use crate::hash_ident;


pub async fn send_read_confirmation(config: &str, ident: &str, redis: &Addr<MyRedisActor>, telegram: &Addr<TelegramActor>) {
    if config.starts_with("telegram:") {
        let chat_id = get_chat_id(&config[9..], redis).await;
        if let Some(chat_id) = chat_id {
            telegram.do_send(SendMessage { chat_id, text: format!("Activity at SecretNote: Your message with ID _{}_ has just been read\\.", escape_markdown(ident)), parse_mode: "MarkdownV2".into() });
            redis.do_send(Command(resp_array!["INCR", "secretnote-stats:telegram-notifications"]));
        } else {
            println!("[Error]: Invalid Telegram config, can't infer chat ID: \"{}\"", config);
        }
    }
}

async fn get_chat_id(recipient: &str, redis: &Addr<MyRedisActor>) -> Option<i64> {
    match recipient.parse::<i64>() {
        Ok(x) => Some(x),
        _ => {
            if recipient.starts_with("@") {
                let result = redis.send(Command(resp_array!["GET", format!("telegram:user_to_chat:{}", &recipient[1..])])).await;
                if let Ok(Ok(RespValue::BulkString(vec))) = result {
                    std::str::from_utf8(&vec).unwrap_or("").parse::<i64>().ok()
                } else { None }
            } else { None }
        }
    }
}

pub async fn check_user_chat_known(recipient: &str, redis: &Addr<MyRedisActor>) -> bool {
    let chat_id = get_chat_id(recipient, redis).await;
    if let Some(chat_id) = chat_id {
        let result = redis.send(Command(resp_array!["SISMEMBER", "telegram:known_chats", format!("{}", chat_id)])).await;
        if let Ok(Ok(RespValue::Integer(result))) = result {
            return result == 1;
        }
    }
    return false;
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

struct FoundAdminLink { admin_ident: String, contains_secret: bool }

fn get_admin_links_from_text(text: &str) -> Vec<FoundAdminLink> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"/note/admin/([A-Za-z0-9_-]{24})(#[A-Za-z0-9_-])?").unwrap();
    }
    return RE.captures_iter(text).map(|m| FoundAdminLink{admin_ident: m[1].into(), contains_secret: m.get(2).is_some()}).collect();
}


pub async fn telegram_message(body: Bytes, redis: web::Data<Addr<MyRedisActor>>, _telegram: web::Data<Addr<TelegramActor>>) -> impl Responder {
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
            println!("Telegram Bot: Received message from @{} in chat {}", username, chat_id);
            redis.do_send(Command(resp_array!["SET", format!("telegram:user_to_chat:{}", username), format!("{}", chat_id)]));
        }
        redis.do_send(Command(resp_array!["SADD", "telegram:known_chats", format!("{}", chat_id)]));

        let mut userinfo = if let Some(username) = username {
            format!("Use your *chat ID \"{}\"* or your *username \"@{}\"* ", chat_id, escape_markdown(username))
        } else {
            format!("Use your *chat ID \"{}\"* ", chat_id)
        };
        userinfo.push_str("after storing a message\\.");
        let text = message.get("text").and_then(|txt| txt.as_str()).unwrap_or("");
        if text == "/start" {
            let msg = format!("Welcome to SecretNoteBot \\- you can now receive read notifications for your messages\\!\n{}\nYou can also send your admin links to this bot \\(up to the \\# char\\)\\.", userinfo);
            return HttpResponse::Ok().json(TelegramWebhookMessageResponse {
                method: "sendMessage".into(),
                chat_id,
                text: msg,
                parse_mode: "MarkdownV2".into(),
            });
        }

        // parse links from other messages
        let links = get_admin_links_from_text(text);
        if links.is_empty() {
            let msg = format!("Please send me admin links \\(up to the \\# char\\) to get read notifications\\.\n{}", &userinfo);
            return HttpResponse::Ok().json(TelegramWebhookMessageResponse {
                method: "sendMessage".into(),
                chat_id,
                text: msg,
                parse_mode: "MarkdownV2".into(),
            });
        }
        let mut msg = String::new();
        for admin_link in links {
            let x = redis.send(Command(resp_array!["GET", format!("noteadmin:{}", admin_link.admin_ident)])).await;
            if let Ok(Ok(RespValue::BulkString(_))) = x {
                let ident = hash_ident(&admin_link.admin_ident);
                redis.do_send(Command(resp_array!["SET", format!("note_settings:read_confirmation:{}", &ident), format!("telegram:{}", chat_id), "EX", format!("{}", 3600 * 24 * 7)]));
                msg.push_str(&format!("You'll receive read notifications for message _{}_\\.", escape_markdown(&admin_link.admin_ident)));
            } else {
                msg.push_str(&format!("Message _{}_ does not exist\\.", escape_markdown(&admin_link.admin_ident)));
            }
            if admin_link.contains_secret {
                msg.push_str(&format!(" *Warning:* Do not include your keys in these links \\(don't send everything after the \\# char\\)\\!"))
            }
            msg.push_str("\n");
        }
        msg.push_str(&userinfo);

        return HttpResponse::Ok().json(TelegramWebhookMessageResponse {
            method: "sendMessage".into(),
            chat_id,
            text: msg,
            parse_mode: "MarkdownV2".into(),
        });
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
            println!("Telegram Bot: available, named \"{}\"", &act.username);
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
            if current_url != act.webhook_url {
                // Set webhook to current url
                println!("Telegram Bot: Current webhook: \"{}\", new webhook: \"{}\"", current_url, act.webhook_url);
                ctx.address().do_send(SetWebhook { webhook_url: act.webhook_url.clone() });
            } else {
                println!("Telegram Bot: Webhook already set.");
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
