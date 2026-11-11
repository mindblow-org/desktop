use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use domain::{Event, Listener};
use serde::Serialize;
use serde_json::json;
use tokio::sync::broadcast;

const NAME: usize = 40;

#[derive(Clone, Debug, Serialize)]
pub struct Peer {
    pub id: String,
    pub name: String,
    pub since: i64,
}

struct Seat {
    conns: usize,
    name: String,
    since: i64,
}

pub struct Live {
    tx: broadcast::Sender<Arc<str>>,
    peers: Mutex<HashMap<String, Seat>>,
}

impl Default for Live {
    fn default() -> Self {
        Self::new()
    }
}

impl Live {
    pub fn new() -> Self {
        Self { tx: broadcast::channel(256).0, peers: Mutex::default() }
    }

    pub fn rx(&self) -> broadcast::Receiver<Arc<str>> {
        self.tx.subscribe()
    }

    pub fn join(&self, dev: &str, name: &str) {
        let list = {
            let mut p = self.peers.lock().unwrap();
            let s = p.entry(dev.to_owned()).or_insert_with(|| Seat { conns: 0, name: String::new(), since: now() });
            s.conns += 1;
            let name = clean(name);
            if !name.is_empty() {
                s.name = name;
            }
            list(&p)
        };
        self.count(list);
    }

    pub fn leave(&self, dev: &str) {
        let list = {
            let mut p = self.peers.lock().unwrap();
            if let Some(s) = p.get_mut(dev) {
                s.conns -= 1;
                if s.conns == 0 {
                    p.remove(dev);
                }
            }
            list(&p)
        };
        self.count(list);
    }

    pub fn rename(&self, dev: &str, name: &str) {
        let list = {
            let mut p = self.peers.lock().unwrap();
            match p.get_mut(dev) {
                Some(s) => s.name = clean(name),
                None => return,
            }
            list(&p)
        };
        self.count(list);
    }

    fn count(&self, list: Vec<Peer>) {
        self.send(json!({ "type": "peers", "count": list.len(), "list": list }).to_string());
    }

    fn send(&self, s: String) {
        let _ = self.tx.send(s.into());
    }
}

impl Listener for Live {
    fn on(&self, e: &Event) {
        if let Ok(s) = serde_json::to_string(e) {
            self.send(s);
        }
    }
}

fn list(p: &HashMap<String, Seat>) -> Vec<Peer> {
    let mut v: Vec<_> = p.iter().map(|(id, s)| Peer { id: id.clone(), name: s.name.clone(), since: s.since }).collect();
    v.sort_by(|a, b| a.since.cmp(&b.since).then_with(|| a.id.cmp(&b.id)));
    v
}

fn clean(s: &str) -> String {
    s.trim().chars().filter(|c| !c.is_control()).take(NAME).collect()
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or_default()
}
