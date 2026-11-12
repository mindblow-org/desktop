use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use domain::{Event, Listener};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{broadcast, watch};

const NAME: usize = 40;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

/// Uma mensagem do `/api/live`. O evento de nota tem também a versão curta (tipo e id),
/// para o sync de outro mindblow: ele só quer saber que algo mudou, e a nota vem pelo feed.
#[derive(Clone)]
pub struct Msg {
    full: Arc<str>,
    lean: Option<Arc<str>>,
}

impl Msg {
    pub fn text(&self, lean: bool) -> &str {
        match &self.lean {
            Some(l) if lean => l,
            _ => &self.full,
        }
    }
}

/// Os aparelhos ligados a este e os que os links contam: cada janela daqui é uma
/// cadeira, e cada link traz a lista do aparelho do outro lado.
pub struct Live {
    tx: broadcast::Sender<Msg>,
    peers: Mutex<HashMap<String, Seat>>,
    far: Mutex<HashMap<String, Vec<Peer>>>,
    me: String,
    name: watch::Sender<String>,
}

impl Default for Live {
    fn default() -> Self {
        Self::new("")
    }
}

impl Live {
    /// `me` é o id deste aparelho, para ele não aparecer duas vezes na lista que volta dos links.
    pub fn new(me: &str) -> Self {
        Self {
            tx: broadcast::channel(256).0,
            peers: Mutex::default(),
            far: Mutex::default(),
            me: me.into(),
            name: watch::channel(String::new()).0,
        }
    }

    pub fn rx(&self) -> broadcast::Receiver<Msg> {
        self.tx.subscribe()
    }

    /// Nome deste aparelho para os outros: o da janela mais antiga que tem nome.
    pub fn name(&self) -> watch::Receiver<String> {
        self.name.subscribe()
    }

    pub fn join(&self, dev: &str, name: &str) {
        {
            let mut p = self.peers.lock().unwrap();
            let s = p.entry(dev.to_owned()).or_insert_with(|| Seat { conns: 0, name: String::new(), since: now() });
            s.conns += 1;
            let name = clean(name);
            if !name.is_empty() {
                s.name = name;
            }
        }
        self.count();
    }

    pub fn leave(&self, dev: &str) {
        {
            let mut p = self.peers.lock().unwrap();
            if let Some(s) = p.get_mut(dev) {
                s.conns -= 1;
                if s.conns == 0 {
                    p.remove(dev);
                }
            }
        }
        self.count();
    }

    pub fn rename(&self, dev: &str, name: &str) {
        {
            let mut p = self.peers.lock().unwrap();
            match p.get_mut(dev) {
                Some(s) => s.name = clean(name),
                None => return,
            }
        }
        self.count();
    }

    /// A lista de aparelhos que um link mandou (vazia quando ele cai).
    pub fn far(&self, link: &str, list: Vec<Peer>) {
        {
            let mut f = self.far.lock().unwrap();
            let same = f.get(link).map_or(list.is_empty(), |l| *l == list);
            if same {
                return;
            }
            if list.is_empty() {
                f.remove(link);
            } else {
                f.insert(link.to_owned(), list);
            }
        }
        self.count();
    }

    /// Estado do sync com cada link.
    pub fn links(&self, list: &impl Serialize) {
        self.send(json!({ "type": "sync", "list": list }).to_string());
    }

    pub fn list(&self) -> Vec<Peer> {
        let p = self.peers.lock().unwrap();
        let mut v: Vec<_> = p.iter().map(|(id, s)| Peer { id: id.clone(), name: s.name.clone(), since: s.since }).collect();
        v.sort_by(|a, b| a.since.cmp(&b.since).then_with(|| a.id.cmp(&b.id)));
        let name = v.iter().find(|p| !p.name.is_empty()).map(|p| p.name.clone()).unwrap_or_default();
        self.name.send_if_modified(|n| {
            if *n == name {
                return false;
            }
            *n = name;
            true
        });
        for l in self.far.lock().unwrap().values() {
            for x in l {
                if x.id != self.me && !v.iter().any(|y| y.id == x.id) {
                    v.push(x.clone());
                }
            }
        }
        v
    }

    fn count(&self) {
        let list = self.list();
        self.send(json!({ "type": "peers", "count": list.len(), "list": list }).to_string());
    }

    fn send(&self, s: String) {
        let _ = self.tx.send(Msg { full: s.into(), lean: None });
    }
}

impl Listener for Live {
    fn on(&self, e: &Event) {
        let Ok(full) = serde_json::to_string(e) else { return };
        let lean = match e {
            Event::Created { note } => Some(json!({ "type": "created", "id": note.id })),
            Event::Updated { note } => Some(json!({ "type": "updated", "id": note.id })),
            Event::Deleted { .. } => None,
        };
        let _ = self.tx.send(Msg { full: full.into(), lean: lean.map(|l| l.to_string().into()) });
    }
}

fn clean(s: &str) -> String {
    s.trim().chars().filter(|c| !c.is_control()).take(NAME).collect()
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or_default()
}
