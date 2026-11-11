use std::{sync::Arc, time::Duration};

use futures_util::{SinkExt, StreamExt};
use reqwest::{Client, Method as M, StatusCode, header};
use serde_json::{Value, json};
use server::{App, Config, router};
use store::Sqlite;
use tokio::{net::TcpListener, time::timeout};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

struct Srv {
    base: String,
    http: Client,
}

impl Srv {
    async fn up(token: Option<&str>) -> Self {
        let cfg = Config { token: token.map(Into::into), ..Config::default() };
        let db = Arc::new(Sqlite::memory().unwrap());
        let app = App::new(db.clone(), db, &cfg);
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = l.local_addr().unwrap().port();
        tokio::spawn(async move { axum::serve(l, router(app)).await.unwrap() });
        Self { base: format!("127.0.0.1:{port}"), http: Client::new() }
    }

    fn url(&self, p: &str) -> String {
        format!("http://{}{p}", self.base)
    }

    async fn send(&self, m: reqwest::Method, p: &str, body: Option<Value>) -> (StatusCode, Value) {
        let mut r = self.http.request(m, self.url(p));
        if let Some(b) = body {
            r = r.json(&b);
        }
        let r = r.send().await.unwrap();
        let s = r.status();
        (s, r.json().await.unwrap_or(Value::Null))
    }

    async fn ws(&self, q: &str) -> Ws {
        connect_async(format!("ws://{}/api/live?{q}", self.base)).await.unwrap().0
    }
}

async fn next(ws: &mut Ws, kind: &str) -> Value {
    loop {
        let m = timeout(Duration::from_secs(2), ws.next()).await.expect("sem evento").unwrap().unwrap();
        if let Message::Text(t) = m {
            let v: Value = serde_json::from_str(&t).unwrap();
            if v["type"] == kind {
                return v;
            }
        }
    }
}


#[tokio::test]
async fn external_app_manages_a_todo() {
    let s = Srv::up(None).await;

    let (c, t) = s.send(M::POST, "/api/notes", Some(json!({ "kind": "todo", "title": "Mercado", "items": [{ "text": "café" }] }))).await;
    assert_eq!(c, StatusCode::CREATED);
    let id = t["id"].as_str().unwrap().to_owned();

    let (_, all) = s.send(M::GET, "/api/notes", None).await;
    assert_eq!(all.as_array().unwrap().len(), 1);

    let (c, t) = s.send(M::POST, &format!("/api/notes/{id}/items"), Some(json!({ "text": "pão" }))).await;
    assert_eq!(c, StatusCode::CREATED);
    let pao = t["items"][1]["id"].as_str().unwrap().to_owned();

    let (_, t) = s.send(M::PATCH, &format!("/api/notes/{id}/items/{pao}"), Some(json!({ "done": true }))).await;
    assert_eq!(t["items"][1]["done"], true);

    let (_, t) = s.send(M::PATCH, &format!("/api/notes/{id}"), Some(json!({ "title": "Feira" }))).await;
    assert_eq!(t["title"], "Feira");

    let (c, _) = s.send(M::DELETE, &format!("/api/notes/{id}"), None).await;
    assert_eq!(c, StatusCode::NO_CONTENT);
    let (c, e) = s.send(M::GET, &format!("/api/notes/{id}"), None).await;
    assert_eq!(c, StatusCode::NOT_FOUND);
    assert_eq!(e["error"], "nota não encontrada");
}

#[tokio::test]
async fn wrong_shape_is_explained() {
    let s = Srv::up(None).await;

    let (c, e) = s.send(M::POST, "/api/notes", Some(json!({ "kind": "sticky", "title": "x" }))).await;

    assert_eq!(c, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(e["error"], "sticky não tem título");
}

#[tokio::test]
async fn other_device_sees_changes_live() {
    let s = Srv::up(None).await;
    let mut phone = s.ws("d=celular").await;
    assert_eq!(next(&mut phone, "peers").await["count"], 1);

    let (_, n) = s.send(M::POST, "/api/notes", Some(json!({ "kind": "sticky", "body": "porta 7341" }))).await;
    let e = next(&mut phone, "created").await;
    assert_eq!(e["note"]["body"], "porta 7341");

    let id = n["id"].as_str().unwrap();
    s.send(M::PATCH, &format!("/api/notes/{id}"), Some(json!({ "body": "porta 7342" }))).await;
    assert_eq!(next(&mut phone, "updated").await["note"]["body"], "porta 7342");

    s.send(M::DELETE, &format!("/api/notes/{id}"), None).await;
    assert_eq!(next(&mut phone, "deleted").await["id"], id);
}

#[tokio::test]
async fn peers_count_devices_not_windows() {
    let s = Srv::up(None).await;
    let mut pc = s.ws("d=pc").await;
    next(&mut pc, "peers").await;

    let _popup = s.ws("d=pc").await;
    assert_eq!(next(&mut pc, "peers").await["count"], 1);

    let _phone = s.ws("d=celular").await;
    assert_eq!(next(&mut pc, "peers").await["count"], 2);
}

#[tokio::test]
async fn devices_show_up_by_name() {
    let s = Srv::up(None).await;
    let mut pc = s.ws("d=pc&n=Notebook").await;
    next(&mut pc, "peers").await;

    let mut phone = s.ws("d=celular&n=iPhone%20%C2%B7%20Safari").await;
    let p = next(&mut pc, "peers").await;
    let names: Vec<_> = p["list"].as_array().unwrap().iter().map(|d| d["name"].as_str().unwrap().to_owned()).collect();
    assert_eq!(names, ["Notebook", "iPhone · Safari"]);

    phone.send(Message::Text(r#"{"type":"name","name":"  Celular  "}"#.into())).await.unwrap();
    let p = next(&mut pc, "peers").await;
    assert_eq!(p["list"][1]["id"], "celular");
    assert_eq!(p["list"][1]["name"], "Celular");
}

#[tokio::test]
async fn health_tells_the_tailnet_link() {
    let cfg = Config { link: Some("https://pc.tail1234.ts.net:7341".into()), ..Config::default() };
    let db = Arc::new(Sqlite::memory().unwrap());
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = l.local_addr().unwrap().port();
    tokio::spawn(async move { axum::serve(l, router(App::new(db.clone(), db, &cfg))).await.unwrap() });

    let h: Value = reqwest::get(format!("http://127.0.0.1:{port}/api/health")).await.unwrap().json().await.unwrap();
    assert_eq!(h["link"], "https://pc.tail1234.ts.net:7341");
}

#[tokio::test]
async fn token_locks_the_api() {
    let s = Srv::up(Some("s3gredo")).await;

    let r = s.http.get(s.url("/api/notes")).send().await.unwrap();
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);

    let r = s.http.get(s.url("/api/notes")).bearer_auth("s3gredo").send().await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    assert!(connect_async(format!("ws://{}/api/live", s.base)).await.is_err());
    assert!(connect_async(format!("ws://{}/api/live?token=s3gredo", s.base)).await.is_ok());
}

#[tokio::test]
async fn foreign_sites_are_refused() {
    let s = Srv::up(None).await;

    let r = s.http.get(s.url("/api/notes")).header(header::HOST, "evil.example.com").send().await.unwrap();
    assert_eq!(r.status(), StatusCode::FORBIDDEN);

    let r = s
        .http
        .post(s.url("/api/notes"))
        .header(header::ORIGIN, "https://evil.example.com")
        .json(&json!({ "kind": "sticky", "body": "x" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::FORBIDDEN);

    let r = s
        .http
        .get(s.url("/api/notes"))
        .header(header::ORIGIN, format!("http://{}", s.base))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn tailnet_name_is_accepted() {
    let s = Srv::up(None).await;

    for h in ["maquina.tail1234.ts.net:7341", "maquina", "localhost:7341"] {
        let r = s.http.get(s.url("/api/health")).header(header::HOST, h).send().await.unwrap();
        assert_eq!(r.status(), StatusCode::OK, "{h}");
    }
}

#[tokio::test]
async fn other_apps_search_notes() {
    let s = Srv::up(None).await;
    s.send(M::POST, "/api/notes", Some(json!({ "kind": "sticky", "body": "comprar filtro de café" }))).await;
    s.send(M::POST, "/api/notes", Some(json!({ "kind": "todo", "title": "Mercado", "items": [{ "text": "Café" }, { "text": "pão" }] }))).await;
    s.send(M::POST, "/api/notes", Some(json!({ "kind": "text", "title": "Leituras" }))).await;

    let (_, v) = s.send(M::GET, "/api/notes?q=cafe", None).await;
    assert_eq!(v.as_array().unwrap().len(), 2);

    let (_, v) = s.send(M::GET, "/api/notes?q=mercado%20p%C3%A3o", None).await;
    assert_eq!(v[0]["title"], "Mercado");
    assert_eq!(v.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn pinning_moves_note_first_everywhere() {
    let s = Srv::up(None).await;
    let (_, a) = s.send(M::POST, "/api/notes", Some(json!({ "kind": "sticky", "body": "a" }))).await;
    s.send(M::POST, "/api/notes", Some(json!({ "kind": "sticky", "body": "b" }))).await;
    let mut phone = s.ws("d=celular").await;

    let id = a["id"].as_str().unwrap();
    s.send(M::PATCH, &format!("/api/notes/{id}"), Some(json!({ "pinned": true }))).await;

    assert_eq!(next(&mut phone, "updated").await["note"]["pinned"], true);
    let (_, v) = s.send(M::GET, "/api/notes", None).await;
    assert_eq!(v[0]["body"], "a");
}

async fn upload(s: &Srv, id: &str, mime: &str, data: Vec<u8>, q: &str) -> reqwest::Response {
    s.http
        .post(s.url(&format!("/api/notes/{id}/images{q}")))
        .header(header::CONTENT_TYPE, mime)
        .body(data)
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn photo_from_phone_reaches_other_devices() {
    let s = Srv::up(None).await;
    let (_, t) = s.send(M::POST, "/api/notes", Some(json!({ "kind": "text", "title": "Ideia" }))).await;
    let id = t["id"].as_str().unwrap();
    let mut pc = s.ws("d=pc").await;

    let r = upload(&s, id, "image/webp", b"RIFF fake webp".to_vec(), "?w=1200&h=750").await;
    assert_eq!(r.status(), StatusCode::CREATED);

    let e = next(&mut pc, "updated").await;
    let img = &e["note"]["images"][0];
    assert_eq!((img["w"].as_u64(), img["mime"].as_str()), (Some(1200), Some("image/webp")));

    let r = s.http.get(s.url(&format!("/api/images/{}", img["id"].as_str().unwrap()))).send().await.unwrap();
    assert_eq!(r.headers()[header::CONTENT_TYPE], "image/webp");
    assert!(r.headers()[header::CACHE_CONTROL].to_str().unwrap().contains("immutable"));
    assert_eq!(r.bytes().await.unwrap().as_ref(), b"RIFF fake webp");
}

#[tokio::test]
async fn big_photo_passes_but_svg_does_not() {
    let s = Srv::up(None).await;
    let (_, t) = s.send(M::POST, "/api/notes", Some(json!({ "kind": "todo", "title": "Obra" }))).await;
    let id = t["id"].as_str().unwrap();

    assert_eq!(upload(&s, id, "image/jpeg", vec![7; 3 * 1024 * 1024], "").await.status(), StatusCode::CREATED);
    assert_eq!(upload(&s, id, "image/svg+xml", b"<svg/>".to_vec(), "").await.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(upload(&s, id, "text/html", b"<b>".to_vec(), "").await.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn deleting_note_takes_its_images() {
    let s = Srv::up(None).await;
    let (_, t) = s.send(M::POST, "/api/notes", Some(json!({ "kind": "sticky", "body": "polaroid" }))).await;
    let id = t["id"].as_str().unwrap();
    let n: Value = upload(&s, id, "image/png", b"png".to_vec(), "").await.json().await.unwrap();
    let img = n["images"][0]["id"].as_str().unwrap().to_owned();

    assert_eq!(upload(&s, id, "image/png", b"png".to_vec(), "").await.status(), StatusCode::UNPROCESSABLE_ENTITY);

    s.send(M::DELETE, &format!("/api/notes/{id}"), None).await;
    let r = s.http.get(s.url(&format!("/api/images/{img}"))).send().await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn images_need_token_too() {
    let s = Srv::up(Some("s3gredo")).await;
    let t: Value = s
        .http
        .post(s.url("/api/notes"))
        .bearer_auth("s3gredo")
        .json(&json!({ "kind": "text" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = t["id"].as_str().unwrap();
    let n: Value = s
        .http
        .post(s.url(&format!("/api/notes/{id}/images")))
        .bearer_auth("s3gredo")
        .header(header::CONTENT_TYPE, "image/png")
        .body(b"png".to_vec())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let img = n["images"][0]["id"].as_str().unwrap();

    let r = s.http.get(s.url(&format!("/api/images/{img}"))).send().await.unwrap();
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
    let r = s.http.get(s.url(&format!("/api/images/{img}?token=s3gredo"))).send().await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}
