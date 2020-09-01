use actix_web_actors::ws;
use actix::prelude::*;
use actix::{Actor, Addr, StreamHandler, Handler, AsyncContext, ActorContext, fut};
use std::time::{Duration, Instant};
use crate::chatbroker::{ChatMessageBroker, ConnectCmd, DisconnectCmd, BroadcastCmd, ChatMessage, BroadcastBinaryCmd, BinaryChatMessage};
use actix_redis::{RedisActor, Command};
use redis_async::resp_array;
use crate::format_redis_result;
use futures::FutureExt;


/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(11);



/// websocket connection is long running connection, it is easier to handle with an actor
pub struct ChattingWebSocket {
    hb: Instant,
    channel_name: String,
    broker: Addr<ChatMessageBroker>,
    redis: Addr<RedisActor>,
}

impl Actor for ChattingWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
        self.broker.send(ConnectCmd { addr_text: ctx.address().recipient(), addr_binary: ctx.address().recipient(), session: self.channel_name.clone() })
            .into_actor(self)
            .then(|res, _, ctx| {
                if let Err(_) = res {
                    println!("Broker error as response to connect cmd, stopping connection");
                    ctx.stop()
                }
                fut::ready(())
            })
            .wait(ctx);

        println!("Websocket connected");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.broker.send(DisconnectCmd { addr_text: ctx.address().recipient(), addr_binary: ctx.address().recipient(), session: self.channel_name.clone() })
            .into_actor(self)
            .then(|res, _, ctx| {
                if let Err(_) = res {
                    ctx.stop()
                }
                fut::ready(())
            })
            .wait(ctx);
        println!("Websocket disconnected");
    }
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChattingWebSocket {
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
            Ok(ws::Message::Binary(bin)) => {
                // ctx.binary(bin)
                let vec = bin.to_vec();
                let channel_name = self.channel_name.clone();
                let redis = self.redis.clone();
                let f = self.redis.send(Command(resp_array!["LPUSH", format!("chat:{}", self.channel_name), vec]));
                let f = f.map(move |r| {
                    println!("LPUSH {}", format_redis_result(&r));
                    redis.do_send(Command(resp_array!["EXPIRE", format!("chat:{}", channel_name), format!("{}", 3600 * 24 * 7)]))
                });
                /*let f = f.map(|r|{
                    println!("EXPIRE {}", format_redis_result(&r));
                });*/
                ctx.spawn(f.into_actor(self));
                self.broker.do_send(BroadcastBinaryCmd { session: self.channel_name.clone(), content: bin })
            },
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl Handler<ChatMessage> for ChattingWebSocket {
    type Result = ();

    fn handle(&mut self, msg: ChatMessage, ctx: &mut Self::Context) -> Self::Result {
        println!("receive from broker");
        ctx.text(msg.content);
    }
}

impl Handler<BinaryChatMessage> for ChattingWebSocket {
    type Result = ();

    fn handle(&mut self, msg: BinaryChatMessage, ctx: &mut Self::Context) -> Self::Result {
        println!("receive from broker");
        ctx.binary(msg.content);
    }
}

impl ChattingWebSocket {
    pub fn new(channel_name: String, broker: Addr<ChatMessageBroker>, redis: Addr<RedisActor>) -> Self {
        Self { hb: Instant::now(), channel_name, broker, redis }
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