use actix::{Message, Recipient, Actor, Context, Handler};
use std::collections::{HashSet, HashMap};
use bytes::Bytes;

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub content: String
}

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone, Debug)]
pub struct BinaryChatMessage {
    pub content: Bytes
}

#[derive(Message)]
#[rtype(result = "bool")]
pub struct ConnectCmd {
    pub addr_text: Recipient<ChatMessage>,
    pub addr_binary: Recipient<BinaryChatMessage>,
    pub session: String
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DisconnectCmd {
    pub addr_text: Recipient<ChatMessage>,
    pub addr_binary: Recipient<BinaryChatMessage>,
    pub session: String
}

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone, Debug)]
pub struct BroadcastCmd {
    pub session: String,
    pub content: String
}

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone, Debug)]
pub struct BroadcastBinaryCmd{
    pub session: String,
    pub content: Bytes
}


pub struct ChatMessageBroker {
    text_sessions: HashMap<String, HashSet<Recipient<ChatMessage>>>,
    binary_sessions: HashMap<String, HashSet<Recipient<BinaryChatMessage>>>
}

impl Default for ChatMessageBroker {
    fn default() -> ChatMessageBroker {
        ChatMessageBroker {
            text_sessions: HashMap::new(),
            binary_sessions: HashMap::new(),
        }
    }
}

impl Actor for ChatMessageBroker {
    type Context = Context<Self>;
}

impl Handler<BroadcastCmd> for ChatMessageBroker {
    type Result = ();

    fn handle(&mut self, msg: BroadcastCmd, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(set) = self.text_sessions.get(&msg.session) {
            println!("sending message \"{}\" to {} addresses", msg.content, set.len());
            for addr in set {
                addr.do_send(ChatMessage{ content: msg.content.clone() }).expect("could not send chat message!");
            }
        }
    }
}

impl Handler<BroadcastBinaryCmd> for ChatMessageBroker {
    type Result = ();

    fn handle(&mut self, msg: BroadcastBinaryCmd, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(set) = self.binary_sessions.get(&msg.session) {
            println!("sending message \"{:?}\" to {} addresses", msg.content, set.len());
            for addr in set {
                addr.do_send(BinaryChatMessage{ content: msg.content.clone() });
            }
        }
    }
}

impl Handler<ConnectCmd> for ChatMessageBroker {
    type Result = bool;

    fn handle(&mut self, msg: ConnectCmd, _ctx: &mut Self::Context) -> Self::Result {
        let b1 = self.text_sessions.entry(msg.session.clone()).or_insert_with(HashSet::new).insert(msg.addr_text);
        let b2 = self.binary_sessions.entry(msg.session).or_insert_with(HashSet::new).insert(msg.addr_binary);
        return b1 || b2;
    }
}

impl Handler<DisconnectCmd> for ChatMessageBroker {
    type Result = ();

    fn handle(&mut self, msg: DisconnectCmd, _ctx: &mut Self::Context) -> Self::Result {
        match self.text_sessions.get_mut(&msg.session) {
            Some(set) => {
                set.remove(&msg.addr_text);
                if set.is_empty() {
                    self.text_sessions.remove(&msg.session);
                }
            }
            None => {
                println!("Invalid disconnect");
            }
        }
        match self.binary_sessions.get_mut(&msg.session) {
            Some(set) => {
                set.remove(&msg.addr_binary);
                if set.is_empty() {
                    self.binary_sessions.remove(&msg.session);
                }
            }
            None => {
                println!("Invalid disconnect");
            }
        }
    }
}


