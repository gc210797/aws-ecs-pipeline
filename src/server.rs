use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use std::collections::{HashMap, HashSet};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub id: usize,
    pub msg: String,
    pub room: String,
}

pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: usize,
    pub name: String,
}

pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rooms: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
}

impl Default for ChatServer {
    fn default() -> Self {
        let mut rooms = HashMap::new();
        rooms.insert("Main".to_owned(), HashSet::new());

        Self {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
        }
    }
}

impl ChatServer {
    fn send_msg(&self, room: &str, message: &str, skip_id: usize) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        let _ = addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Somone joined!!");

        self.send_msg(&"Main".to_owned(), "Someone Joined!!", 0);

        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        self.rooms
            .entry("Main".to_owned())
            .or_insert(HashSet::new())
            .insert(id);

        id
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected!!");

        let mut rooms: Vec<String> = Vec::new();

        if self.sessions.remove(&msg.id).is_some() {
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }

        for room in rooms {
            self.send_msg(&room, "Some disconnected", 0);
        }
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_msg(&msg.room, msg.msg.as_str(), msg.id);
    }
}

impl Handler<ListRooms> for ChatServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for key in self.rooms.keys() {
            rooms.push(key.to_owned());
        }

        MessageResult(rooms)
    }
}

impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let mut rooms = Vec::new();

        for (n, sessions) in &mut self.rooms {
            if sessions.remove(&msg.id) {
                rooms.push(n.to_owned());
            }
        }

        self.rooms
            .entry(msg.name.clone())
            .or_insert(HashSet::new())
            .insert(msg.id);

        self.send_msg(&msg.name, "Someone connected", msg.id);
    }
}
