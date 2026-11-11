use std::{future::Future, sync::Arc, time::Duration};

use futures_util::StreamExt;
use reqwest::{Client, Method as M, StatusCode};
use serde_json::{Value, json};
use server::{App, Config, serve};
use store::Sqlite;
use tokio::{net::TcpListener, time::sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Um aparelho inteiro: banco em memória, servidor e sync.
struct Node {
    url: String,
    http: Client,
    token: Option<String>,
}

impl Node {
    async fn up(token: Option<&str>) -> Self {
        Self::on(TcpListener::bind("127.0.0.1:0").await.unwrap(), token).await
    }

    async fn on(l: TcpListener, token: Option<&str>) -> Self {
        let cfg = Config { token: token.map(Into::into), ..Config::default() };
        let db = Arc::new(Sqlite::memory().unwrap());
        let app = App::new(db.clone(), db.clone(), db, &cfg).unwrap();
        let url = format!("http://127.0.0.1:{}", l.local_addr().unwrap().port());
        tokio::spawn(async move { serve(app, &cfg, l, std::future::pending()).await.unwrap() });
        Self { url, http: Client::new(), token: token.map(Into::into) }
    }

    async fn call(&self, m: M, p: &str, body: Option<Value>) -> (StatusCode, Value) {
        let mut r = self.http.request(m, format!("{}{p}", self.url));
        if let Some(t) = &self.token {
            r = r.bearer_auth(t);
        }
        if let Some(b) = body {
            r = r.json(&b);
        }
        let r = r.send().await.unwrap();
        let s = r.status();
        (s, r.json().await.unwrap_or(Value::Null))
    }

    async fn add(&self, body: Value) -> String {
        let (c, n) = self.call(M::POST, "/api/notes", Some(body)).await;
        assert_eq!(c, StatusCode::CREATED);
        n["id"].as_str().unwrap().to_owned()
    }

    async fn link(&self, url: &str) {
        let (c, _) = self.call(M::POST, "/api/links", Some(json!({ "url": url }))).await;
        assert_eq!(c, StatusCode::CREATED);
    }

    async fn notes(&self) -> Vec<Value> {
        self.call(M::GET, "/api/notes", None).await.1.as_array().cloned().unwrap_or_default()
    }

    async fn note(&self, id: &str) -> Option<Value> {
        let (c, n) = self.call(M::GET, &format!("/api/notes/{id}"), None).await;
        (c == StatusCode::OK).then_some(n)
    }

    async fn state(&self) -> String {
        let (_, v) = self.call(M::GET, "/api/links", None).await;
        v[0]["state"].as_str().unwrap_or_default().to_owned()
    }
}

/// Espera a condição valer, por até 15 s.
async fn until<F: Future<Output = bool>>(what: &str, mut f: impl FnMut() -> F) {
    for _ in 0..300 {
        if f().await {
            return;
        }
        sleep(Duration::from_millis(50)).await;
    }
    panic!("não aconteceu: {what}");
}

#[tokio::test]
async fn notes_go_both_ways() {
    let pc = Node::up(None).await;
    let phone = Node::up(None).await;
    phone.link(&pc.url).await;

    let id = phone.add(json!({ "kind": "todo", "title": "Mercado", "items": [{ "text": "café" }] })).await;
    until("nota do celular no pc", || async { pc.note(&id).await.is_some() }).await;

    pc.call(M::PATCH, &format!("/api/notes/{id}"), Some(json!({ "title": "Feira" }))).await;
    until("título do pc no celular", || async { phone.note(&id).await.is_some_and(|n| n["title"] == "Feira") }).await;

    pc.call(M::DELETE, &format!("/api/notes/{id}"), None).await;
    until("exclusão do pc no celular", || async { phone.notes().await.is_empty() }).await;
    assert_eq!(phone.state().await, "ok");
}

#[tokio::test]
async fn phone_works_offline_and_catches_up() {
    let spot = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://127.0.0.1:{}", spot.local_addr().unwrap().port());
    drop(spot);
    let phone = Node::up(None).await;
    phone.link(&url).await;

    let id = phone.add(json!({ "kind": "sticky", "body": "anotado no metrô" })).await;
    phone.call(M::PATCH, &format!("/api/notes/{id}"), Some(json!({ "pinned": true }))).await;
    until("link offline", || async { phone.state().await == "off" }).await;

    let pc = Node::on(TcpListener::bind(&url[7..]).await.unwrap(), None).await;
    until("nota offline no pc", || async { pc.note(&id).await.is_some_and(|n| n["pinned"] == true) }).await;
    until("link de volta", || async { phone.state().await == "ok" }).await;
}

#[tokio::test]
async fn edits_on_both_sides_merge() {
    let pc = Node::up(None).await;
    let phone = Node::up(None).await;
    let id = pc.add(json!({ "kind": "text", "title": "Ideia", "body": "grid" })).await;
    phone.link(&pc.url).await;
    until("nota do pc no celular", || async { phone.note(&id).await.is_some() }).await;

    phone.call(M::DELETE, &format!("/api/links?url={}", pc.url), None).await;
    phone.call(M::PATCH, &format!("/api/notes/{id}"), Some(json!({ "title": "Ideia boa" }))).await;
    pc.call(M::PATCH, &format!("/api/notes/{id}"), Some(json!({ "body": "grid simples" }))).await;
    phone.link(&pc.url).await;

    let both = |n: Option<Value>| n.is_some_and(|n| n["title"] == "Ideia boa" && n["body"] == "grid simples");
    until("junção no pc", || async { both(pc.note(&id).await) }).await;
    until("junção no celular", || async { both(phone.note(&id).await) }).await;
}

#[tokio::test]
async fn photo_travels_with_the_note() {
    let pc = Node::up(None).await;
    let phone = Node::up(None).await;
    phone.link(&pc.url).await;
    let id = phone.add(json!({ "kind": "text", "title": "Foto" })).await;
    let png = b"\x89PNG fake".to_vec();
    let r = phone.http.post(format!("{}/api/notes/{id}/images?w=4&h=3", phone.url)).header("content-type", "image/png").body(png.clone()).send().await.unwrap();
    let img = r.json::<Value>().await.unwrap()["images"][0]["id"].as_str().unwrap().to_owned();

    until("imagem no pc", || async {
        let r = pc.http.get(format!("{}/api/images/{img}", pc.url)).send().await.unwrap();
        r.status() == StatusCode::OK && r.bytes().await.unwrap().to_vec() == png
    })
    .await;
}

#[tokio::test]
async fn token_from_the_qr_opens_the_hub() {
    let pc = Node::up(Some("s3gredo")).await;
    let phone = Node::up(None).await;

    phone.link(&format!("{}/#token=errado", pc.url)).await;
    until("token recusado", || async { phone.state().await == "locked" }).await;

    phone.link(&format!("{}/#token=s3gredo", pc.url)).await;
    let id = phone.add(json!({ "kind": "sticky", "body": "oi" })).await;
    until("nota no pc com token", || async { pc.note(&id).await.is_some() }).await;
}

#[tokio::test]
async fn own_link_is_noticed() {
    let phone = Node::up(None).await;
    phone.link(&phone.url).await;

    until("link para si mesmo", || async { phone.state().await == "self" }).await;
}

#[tokio::test]
async fn devices_on_the_hub_show_up_on_the_phone() {
    let pc = Node::up(None).await;
    let phone = Node::up(None).await;
    let host = |n: &Node| n.url.trim_start_matches("http://").to_owned();
    let (mut desk, _) = connect_async(format!("ws://{}/api/live?d=pc-janela&n=Notebook", host(&pc))).await.unwrap();
    let (mut app, _) = connect_async(format!("ws://{}/api/live?d=tela&n=Celular", host(&phone))).await.unwrap();
    phone.link(&pc.url).await;

    let names = |list: &Value| list.as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap().to_owned()).collect::<Vec<_>>();
    loop {
        let Some(Ok(Message::Text(t))) = desk.next().await else { panic!("sem eventos no pc") };
        let v: Value = serde_json::from_str(&t).unwrap();
        if v["type"] == "peers" && names(&v["list"]) == ["Notebook", "Celular"] {
            break;
        }
    }
    loop {
        let Some(Ok(Message::Text(t))) = app.next().await else { panic!("sem eventos no celular") };
        let v: Value = serde_json::from_str(&t).unwrap();
        if v["type"] == "peers" && names(&v["list"]) == ["Celular", "Notebook"] {
            break;
        }
    }
}
