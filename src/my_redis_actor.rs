use std::collections::VecDeque;
use std::io;

use actix::actors::resolver::{Connect, Resolver};
use actix::prelude::*;
use actix_utils::oneshot;
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use futures::{FutureExt};
use redis_async::error::Error as RespError;
use redis_async::resp::{RespCodec, RespValue};
use redis_async::resp_array;
use tokio::io::{split, WriteHalf};
use tokio::net::TcpStream;
use tokio_util::codec::FramedRead;
use actix_redis::{Error, Command};
use log::{info, warn, error};


/// Redis comminucation actor
pub struct MyRedisActor {
    addr: String,
    db: Option<u32>,
    auth: Option<String>,
    backoff: ExponentialBackoff,
    cell: Option<actix::io::FramedWrite<RespValue, WriteHalf<TcpStream>, RespCodec>>,
    queue: VecDeque<oneshot::Sender<Result<RespValue, Error>>>,
}

impl MyRedisActor {
    /// Start new `Supervisor` with `MyRedisActor`.
    pub fn start<S: Into<String>>(addr: S, db: Option<u32>, auth: Option<String>) -> Addr<MyRedisActor> {
        let addr = addr.into();

        let mut backoff = ExponentialBackoff::default();
        backoff.max_elapsed_time = None;

        Supervisor::start(move |_| MyRedisActor {
            addr,
            db,
            auth,
            cell: None,
            backoff,
            queue: VecDeque::new(),
        })
    }
}

impl Actor for MyRedisActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        Resolver::from_registry()
            .send(Connect::host(self.addr.as_str()))
            .into_actor(self)
            .map(|res, act, ctx| match res {
                Ok(res) => match res {
                    Ok(stream) => {
                        info!("Connected to redis server: {}", act.addr);

                        let (r, w) = split(stream);

                        // configure write side of the connection
                        let framed = actix::io::FramedWrite::new(w, RespCodec, ctx);
                        act.cell = Some(framed);

                        // read side of the connection
                        ctx.add_stream(FramedRead::new(r, RespCodec));

                        act.backoff.reset();

                        if let Some(a) = &act.auth {
                            ctx.address().do_send(Command(resp_array!["AUTH", a]));
                        }
                        if let Some(db) = act.db {
                            ctx.address().do_send(Command(resp_array!["SELECT", format!("{}", db)]));
                        }
                    }
                    Err(err) => {
                        error!("Can not connect to redis server: {}", err);
                        // re-connect with backoff time.
                        // we stop current context, supervisor will restart it.
                        if let Some(timeout) = act.backoff.next_backoff() {
                            ctx.run_later(timeout, |_, ctx| ctx.stop());
                        }
                    }
                },
                Err(err) => {
                    error!("Can not connect to redis server: {}", err);
                    // re-connect with backoff time.
                    // we stop current context, supervisor will restart it.
                    if let Some(timeout) = act.backoff.next_backoff() {
                        ctx.run_later(timeout, |_, ctx| ctx.stop());
                    }
                }
            }).wait(ctx);
    }
}

impl Supervised for MyRedisActor {
    fn restarting(&mut self, _: &mut Self::Context) {
        self.cell.take();
        for tx in self.queue.drain(..) {
            let _ = tx.send(Err(Error::Disconnected));
        }
    }
}

impl actix::io::WriteHandler<io::Error> for MyRedisActor {
    fn error(&mut self, err: io::Error, _: &mut Self::Context) -> Running {
        warn!("Redis connection dropped: {} error: {}", self.addr, err);
        Running::Stop
    }
}

impl StreamHandler<Result<RespValue, RespError>> for MyRedisActor {
    fn handle(&mut self, msg: Result<RespValue, RespError>, ctx: &mut Self::Context) {
        match msg {
            Err(e) => {
                if let Some(tx) = self.queue.pop_front() {
                    let _ = tx.send(Err(e.into()));
                }
                ctx.stop();
            }
            Ok(val) => {
                if let Some(tx) = self.queue.pop_front() {
                    let _ = tx.send(Ok(val));
                }
            }
        }
    }
}

impl Handler<Command> for MyRedisActor {
    type Result = ResponseFuture<Result<RespValue, Error>>;

    fn handle(&mut self, msg: Command, _: &mut Self::Context) -> Self::Result {
        let (tx, rx) = oneshot::channel();
        if let Some(ref mut cell) = self.cell {
            self.queue.push_back(tx);
            cell.write(msg.0);
        } else {
            let _ = tx.send(Err(Error::NotConnected));
        }

        Box::pin(rx.map(|res| match res {
            Ok(res) => res,
            Err(_) => Err(Error::Disconnected),
        }))
    }
}
