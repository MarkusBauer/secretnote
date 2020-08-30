use actix_web_actors::ws;
use actix::prelude::*;
use actix::{Actor, Addr, StreamHandler, Handler, AsyncContext, ActorContext, fut};
use std::time::{Duration, Instant};
use crate::chatbroker::{ChatMessageBroker, ConnectCmd, DisconnectCmd, BroadcastCmd, ChatMessage};


/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(11);



/// websocket connection is long running connection, it is easier to handle with an actor
pub struct ChattingWebSocket {
    hb: Instant,
    channel_name: String,
    broker: Addr<ChatMessageBroker>,
}

impl Actor for ChattingWebSocket {
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
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
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
        ctx.text(format!("Test message: {} from {}", msg.content, self.channel_name));
    }
}

impl ChattingWebSocket {
    pub fn new(channel_name: String, broker: Addr<ChatMessageBroker>) -> Self {
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