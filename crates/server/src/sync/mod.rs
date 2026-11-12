mod wire;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use domain::{Event, Id, Link, Links, Listener, Notes};
use futures_util::{SinkExt, StreamExt};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{sync::Notify, task::AbortHandle, time::sleep};
use tokio_tungstenite::tungstenite::Message;

pub use wire::Page;
use wire::{Fail, Wire};

use crate::{Live, live::Peer};

/// Mudanças por vez em cada sentido.
pub const PAGE: usize = 50;
/// Espera depois de um aviso, para juntar várias mudanças num sync só.
const SETTLE: Duration = Duration::from_millis(300);
/// Sem aviso nenhum, sincroniza de novo depois disso.
const IDLE: Duration = Duration::from_secs(60);
/// Primeira espera depois de uma falha; dobra até `MAX`.
const MIN: Duration = Duration::from_secs(1);
const MAX: Duration = Duration::from_secs(30);
/// Outro lado sem sync: tenta de novo bem depois, vai que atualizaram.
const LATER: Duration = Duration::from_secs(300);

/// Como está o sync com um link, para a interface.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub url: String,
    pub token: String,
    /// `wait` (ainda não tentou), `ok`, `off` (não respondeu), `old` (mindblow sem sync),
    /// `locked` (token não serve), `self` (é este aparelho) ou `fail`.
    pub state: String,
    /// Último sync que deu certo.
    pub at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

struct Run {
    kick: Arc<Notify>,
    dial: Arc<Notify>,
    tasks: Vec<AbortHandle>,
    state: State,
}

/// Sincroniza este aparelho com os links que ele conhece: puxa o feed de cada um,
/// empurra o daqui e troca as imagens que faltam. Cada link tem um laço próprio,
/// acordado por mudança daqui (pelo `Bus`) ou de lá (pelo `/api/live` de lá).
pub struct Syncer {
    notes: Arc<Notes>,
    links: Arc<dyn Links>,
    live: Arc<Live>,
    node: Id,
    wire: Wire,
    runs: Mutex<HashMap<String, Run>>,
}

impl Syncer {
    pub fn new(notes: Arc<Notes>, links: Arc<dyn Links>, live: Arc<Live>, node: Id) -> Self {
        Self { notes, links, live, node, wire: Wire::new(), runs: Mutex::default() }
    }

    /// Liga o sync com os links guardados. Precisa estar dentro do runtime do tokio.
    pub fn start(self: &Arc<Self>) -> domain::Result<()> {
        for l in self.links.links()? {
            self.run(l);
        }
        Ok(())
    }

    /// Novo link, ou token novo para um que já existe. Recomeça o sync com ele.
    pub fn add(self: &Arc<Self>, url: &str, token: &str) -> domain::Result<()> {
        let l = Link::new(url, token);
        self.links.join(&l)?;
        let l = self.links.links()?.into_iter().find(|x| x.url == l.url).unwrap_or(l);
        self.run(l);
        Ok(())
    }

    pub fn remove(&self, url: &str) -> domain::Result<bool> {
        let had = self.links.part(url)?;
        if let Some(r) = self.runs.lock().unwrap().remove(url) {
            r.tasks.iter().for_each(AbortHandle::abort);
        }
        self.live.far(url, Vec::new());
        self.tell();
        Ok(had)
    }

    /// A rede do aparelho mudou: o `/api/live` de cada link reconecta na hora, e reconectar acorda o laço.
    pub fn now(&self) {
        for r in self.runs.lock().unwrap().values() {
            r.dial.notify_one();
        }
    }

    pub fn list(&self) -> Vec<State> {
        let mut v: Vec<_> = self.runs.lock().unwrap().values().map(|r| r.state.clone()).collect();
        v.sort_by(|a, b| a.url.cmp(&b.url));
        v
    }

    fn run(self: &Arc<Self>, l: Link) {
        let kick = Arc::new(Notify::new());
        let up = Arc::new(Notify::new());
        let dial = Arc::new(Notify::new());
        let state = State { url: l.url.clone(), token: l.token.clone(), state: "wait".into(), at: None, error: None };
        let tasks = vec![
            tokio::spawn(self.clone().turn(l.url.clone(), kick.clone(), up.clone())).abort_handle(),
            tokio::spawn(self.clone().listen(l.url.clone(), kick.clone(), up, dial.clone())).abort_handle(),
        ];
        let old = self.runs.lock().unwrap().insert(l.url.clone(), Run { kick, dial, tasks, state });
        if let Some(r) = old {
            r.tasks.iter().for_each(AbortHandle::abort);
        }
        self.tell();
    }

    /// O laço de um link: sincroniza, espera um aviso (ou o tempo) e repete.
    async fn turn(self: Arc<Self>, url: String, kick: Arc<Notify>, up: Arc<Notify>) {
        let mut wait = MIN;
        let mut first = true;
        loop {
            match self.round(&url, first).await {
                Ok(()) => {
                    first = false;
                    wait = MIN;
                    self.set(&url, "ok", None);
                    tokio::select! {
                        _ = kick.notified() => sleep(SETTLE).await,
                        _ = sleep(IDLE) => {}
                    }
                }
                Err(Fail::Me) => return self.halt(&url, "self"),
                Err(Fail::Locked) => return self.halt(&url, "locked"),
                Err(Fail::Old) => {
                    self.set(&url, "old", None);
                    sleep(LATER).await;
                }
                Err(Fail::Off(e) | Fail::Bad(e)) if self.gone(&url) => return eprintln!("mindblow: sync {url}: {e}"),
                Err(f) => {
                    let (s, e) = match f {
                        Fail::Off(e) => ("off", e),
                        Fail::Bad(e) => ("fail", e),
                        _ => unreachable!(),
                    };
                    self.set(&url, s, Some(e));
                    // Offline, mudança daqui não adianta tentar de novo; o /api/live de lá voltar, sim.
                    tokio::select! {
                        _ = up.notified() => {}
                        _ = sleep(wait) => {}
                    }
                    wait = (wait * 2).min(MAX);
                }
            }
        }
    }

    /// Um sync completo com o link: primeiro puxa, depois empurra.
    async fn round(&self, url: &str, first: bool) -> Result<(), Fail> {
        let Some(mut l) = self.link(url)? else { return Err(Fail::Bad("link removido".into())) };
        loop {
            let p = self.wire.page(&l, l.pull, PAGE).await?;
            if p.node == self.node {
                return Err(Fail::Me);
            }
            let (seq, more) = (p.feed.seq, p.feed.more);
            let t = self.notes.take(p.feed.notes, p.feed.gone)?;
            // Nada daqui esperando: o que entrou agora não precisa voltar para lá.
            if t.from == l.push {
                l.push = t.to;
            }
            l.pull = seq;
            self.links.moved(&l.url, l.pull, l.push)?;
            self.fetch(&l, t.need).await?;
            if !more {
                break;
            }
        }
        if first {
            self.fetch(&l, self.notes.lack()?).await?;
        }
        loop {
            let f = self.notes.feed(l.push, PAGE)?;
            if f.notes.is_empty() && f.gone.is_empty() {
                break;
            }
            let (seq, more) = (f.seq, f.more);
            let t = self.wire.send(&l, f.notes, f.gone).await?;
            for id in t.need {
                if let Ok((mime, data)) = self.notes.image(id) {
                    self.wire.put(&l, id, &mime, data).await?;
                }
            }
            if t.from == l.pull {
                l.pull = t.to;
            }
            l.push = seq;
            self.links.moved(&l.url, l.pull, l.push)?;
            if !more {
                break;
            }
        }
        Ok(())
    }

    async fn fetch(&self, l: &Link, need: Vec<Id>) -> Result<(), Fail> {
        for id in need {
            if let Some((mime, data)) = self.wire.image(l, id).await? {
                self.notes.adopt(id, &mime, &data)?;
            }
        }
        Ok(())
    }

    /// Fica ligado no `/api/live` do link: mudança lá acorda o laço, e a lista de
    /// aparelhos de lá entra na daqui. Também leva o nome deste aparelho.
    async fn listen(self: Arc<Self>, url: String, kick: Arc<Notify>, up: Arc<Notify>, dial: Arc<Notify>) {
        let mut wait = MIN;
        loop {
            let Ok(Some(l)) = self.link(&url) else { return };
            let mut names = self.live.name();
            let name = names.borrow_and_update().clone();
            match self.wire.live(&l, self.node, &name).await {
                Ok(ws) => {
                    wait = MIN;
                    up.notify_one();
                    kick.notify_one();
                    let (mut tx, mut rx) = ws.split();
                    loop {
                        tokio::select! {
                            m = rx.next() => match m {
                                Some(Ok(Message::Text(t))) => self.hear(&url, &t, &kick),
                                Some(Ok(_)) => {}
                                _ => break,
                            },
                            Ok(()) = names.changed() => {
                                let n = names.borrow_and_update().clone();
                                let m = json!({ "type": "name", "name": n }).to_string();
                                if tx.send(Message::Text(m.into())).await.is_err() {
                                    break;
                                }
                            }
                            // Depois de trocar de rede, a conexão antiga pode estar morta sem ninguém saber.
                            _ = dial.notified() => break,
                        }
                    }
                    self.live.far(&url, Vec::new());
                }
                Err(Fail::Locked) => return,
                Err(_) => {}
            }
            tokio::select! {
                _ = dial.notified() => wait = MIN,
                _ = sleep(wait) => wait = (wait * 2).min(MAX),
            }
        }
    }

    fn hear(&self, url: &str, t: &str, kick: &Notify) {
        #[derive(Deserialize)]
        struct Said {
            r#type: String,
            #[serde(default)]
            list: Vec<Peer>,
        }
        let Ok(s) = serde_json::from_str::<Said>(t) else { return };
        match s.r#type.as_str() {
            "created" | "updated" | "deleted" | "resync" => kick.notify_one(),
            "peers" => self.live.far(url, s.list),
            _ => {}
        }
    }

    fn link(&self, url: &str) -> domain::Result<Option<Link>> {
        Ok(self.links.links()?.into_iter().find(|l| l.url == url))
    }

    fn gone(&self, url: &str) -> bool {
        self.link(url).is_ok_and(|l| l.is_none())
    }

    fn set(&self, url: &str, state: &str, error: Option<String>) {
        {
            let mut runs = self.runs.lock().unwrap();
            let Some(r) = runs.get_mut(url) else { return };
            let s = &mut r.state;
            if state == "ok" {
                s.at = Some(now());
            }
            if s.state == state && s.error == error && state != "ok" {
                return;
            }
            s.state = state.into();
            s.error = error;
        }
        self.tell();
    }

    /// Para de tentar: o link é este aparelho, ou o token não serve (até vir outro).
    fn halt(&self, url: &str, state: &str) {
        self.set(url, state, None);
        if let Some(r) = self.runs.lock().unwrap().get(url) {
            r.tasks.iter().for_each(AbortHandle::abort);
        }
        self.live.far(url, Vec::new());
    }

    fn tell(&self) {
        self.live.links(&self.list());
    }
}

impl Listener for Syncer {
    fn on(&self, _: &Event) {
        for r in self.runs.lock().unwrap().values() {
            r.kick.notify_one();
        }
    }
}

/// Lê o link que a pessoa colou ou o QR trouxe: fica só a origem, e o `#token=` vira o token.
pub fn parse(raw: &str) -> Option<(String, String)> {
    let raw = raw.trim();
    let u = Url::parse(raw)
        .ok()
        .filter(|u| u.has_host())
        .or_else(|| Url::parse(&format!("https://{raw}")).ok())?;
    if !matches!(u.scheme(), "http" | "https") || u.host_str().is_none_or(str::is_empty) {
        return None;
    }
    let token = u
        .fragment()
        .and_then(|f| f.split('&').find_map(|kv| kv.strip_prefix("token=")))
        .map(unescape)
        .unwrap_or_default();
    Some((u.origin().ascii_serialization(), token))
}

fn unescape(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = b
            .get(i + 1..i + 3)
            .filter(|_| b[i] == b'%')
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match hex {
            Some(c) => {
                out.push(c);
                i += 3;
            }
            None => {
                out.push(b[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn link_from_qr_keeps_origin_and_token() {
        assert_eq!(
            parse("https://pc.tail1234.ts.net:7341/#token=a%2Bb").unwrap(),
            ("https://pc.tail1234.ts.net:7341".into(), "a+b".into())
        );
    }

    #[test]
    fn typed_link_gets_https() {
        assert_eq!(parse(" pc.tail1234.ts.net:7341 ").unwrap().0, "https://pc.tail1234.ts.net:7341");
        assert_eq!(parse("http://10.0.2.2:7341/n/abc").unwrap(), ("http://10.0.2.2:7341".into(), String::new()));
    }

    #[test]
    fn junk_is_refused() {
        assert!(parse("").is_none());
        assert!(parse("ftp://pc").is_none());
    }
}
