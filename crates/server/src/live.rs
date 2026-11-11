use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use domain::{Event, Listener};
use serde_json::json;
use tokio::sync::broadcast;

pub struct Live {
    tx: broadcast::Sender<Arc<str>>,
    peers: Mutex<HashMap<String, usize>>,
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

    pub fn join(&self, dev: &str) {
        let n = {
            let mut p = self.peers.lock().unwrap();
            *p.entry(dev.to_owned()).or_default() += 1;
            p.len()
        };
        self.count(n);
    }

    pub fn leave(&self, dev: &str) {
        let n = {
            let mut p = self.peers.lock().unwrap();
            if let Some(c) = p.get_mut(dev) {
                *c -= 1;
                if *c == 0 {
                    p.remove(dev);
                }
            }
            p.len()
        };
        self.count(n);
    }

    fn count(&self, n: usize) {
        self.send(json!({ "type": "peers", "count": n }).to_string());
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
