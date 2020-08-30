use actix::{Message, Recipient, Actor, Context, Handler};
use std::collections::{HashSet, HashMap};

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub content: String
}

#[derive(Message)]
#[rtype(result = "bool")]
pub struct ConnectCmd {
    pub addr: Recipient<ChatMessage>,
    pub session: String
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DisconnectCmd {
    pub addr: Recipient<ChatMessage>,
    pub session: String
}

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone, Debug)]
pub struct BroadcastCmd {
    pub session: String,
    pub content: String
}


pub struct ChatMessageBroker {
    sessions: HashMap<String, HashSet<Recipient<ChatMessage>>>
}

impl Default for ChatMessageBroker {
    fn default() -> ChatMessageBroker {
        ChatMessageBroker {
            sessions: HashMap::new()
        }
    }
}

impl Actor for ChatMessageBroker {
    type Context = Context<Self>;
}

impl Handler<BroadcastCmd> for ChatMessageBroker {
    type Result = ();

    fn handle(&mut self, msg: BroadcastCmd, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(set) = self.sessions.get(&msg.session) {
            println!("sending message \"{}\" to {} addresses", msg.content, set.len());
            for addr in set {
                addr.do_send(ChatMessage{ content: msg.content.clone() }).expect("could not send chat message!");
            }
        }
    }
}

impl Handler<ConnectCmd> for ChatMessageBroker {
    type Result = bool;

    fn handle(&mut self, msg: ConnectCmd, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.entry(msg.session).or_insert_with(HashSet::new).insert(msg.addr)
    }
}

impl Handler<DisconnectCmd> for ChatMessageBroker {
    type Result = ();

    fn handle(&mut self, msg: DisconnectCmd, _ctx: &mut Self::Context) -> Self::Result {
        match self.sessions.get_mut(&msg.session) {
            Some(set) => {
                set.remove(&msg.addr);
                if set.is_empty() {
                    self.sessions.remove(&msg.session);
                }
            }
            None => {
                println!("Invalid disconnect")
            }
        }
    }
}


