use std::{sync::Arc, time::Duration};

use domain::{Feed, Gone, Id, Link, Shot, Took};
use reqwest::{Client, RequestBuilder, Response, StatusCode, Url, header::CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream, connect_async_tls_with_config};

/// Uma página do feed de outro aparelho, com o id dele.
#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub node: Id,
    #[serde(flatten)]
    pub feed: Feed,
}

/// Por que um sync não deu certo.
#[derive(Debug)]
pub enum Fail {
    /// O outro aparelho não respondeu (offline, sem tailnet, servidor fora do ar).
    Off(String),
    /// Responde, mas é um mindblow sem sync.
    Old,
    /// Pediu token, e o que temos não serve.
    Locked,
    /// O link é deste próprio aparelho.
    Me,
    /// Qualquer outro erro, daqui ou de lá.
    Bad(String),
}

impl From<domain::Error> for Fail {
    fn from(e: domain::Error) -> Self {
        Fail::Bad(e.to_string())
    }
}

pub type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// HTTP e WebSocket para os outros aparelhos.
pub struct Wire {
    http: Client,
    tls: Arc<rustls::ClientConfig>,
}

impl Wire {
    pub fn new() -> Self {
        let tls = tls();
        let http = Client::builder()
            .tls_backend_preconfigured((*tls).clone())
            .connect_timeout(Duration::from_secs(8))
            .timeout(Duration::from_secs(120))
            .build()
            .expect("cliente http");
        Self { http, tls }
    }

    pub async fn page(&self, l: &Link, since: i64, limit: usize) -> Result<Page, Fail> {
        let r = self.auth(l, self.http.get(format!("{}/api/sync?since={since}&limit={limit}", l.url)));
        sync(r).await?.json().await.map_err(off)
    }

    pub async fn send(&self, l: &Link, notes: Vec<Shot>, gone: Vec<Gone>) -> Result<Took, Fail> {
        let r = self.auth(l, self.http.post(format!("{}/api/sync", l.url)).json(&json!({ "notes": notes, "gone": gone })));
        sync(r).await?.json().await.map_err(off)
    }

    /// Os bytes de uma imagem, ou nada se o outro lado também não tem.
    pub async fn image(&self, l: &Link, id: Id) -> Result<Option<(String, Vec<u8>)>, Fail> {
        let r = self.auth(l, self.http.get(format!("{}/api/images/{id}", l.url))).send().await.map_err(off)?;
        if r.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let r = ok(r).await?;
        let mime = r.headers().get(CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or_default().to_owned();
        Ok(Some((mime, r.bytes().await.map_err(off)?.to_vec())))
    }

    pub async fn put(&self, l: &Link, id: Id, mime: &str, data: Vec<u8>) -> Result<(), Fail> {
        let r = self.http.put(format!("{}/api/images/{id}", l.url)).header(CONTENT_TYPE, mime).body(data);
        ok(self.auth(l, r).send().await.map_err(off)?).await?;
        Ok(())
    }

    /// O `/api/live` do outro lado: avisa quando algo muda lá e diz quem está ligado.
    pub async fn live(&self, l: &Link, node: Id, name: &str) -> Result<Ws, Fail> {
        let mut u = Url::parse(&l.url).map_err(|e| Fail::Bad(e.to_string()))?;
        let ws = if u.scheme() == "https" { "wss" } else { "ws" };
        u.set_scheme(ws).map_err(|_| Fail::Bad("link inválido".into()))?;
        u.set_path("/api/live");
        {
            let mut q = u.query_pairs_mut();
            q.append_pair("d", &node.to_string());
            q.append_pair("n", name);
            q.append_pair("lean", "true");
            if !l.token.is_empty() {
                q.append_pair("token", &l.token);
            }
        }
        let tls = Some(Connector::Rustls(self.tls.clone()));
        let (s, _) = tokio::time::timeout(Duration::from_secs(10), connect_async_tls_with_config(u.as_str(), None, false, tls))
            .await
            .map_err(|_| Fail::Off("tempo esgotado".into()))?
            .map_err(|e| match e {
                tokio_tungstenite::tungstenite::Error::Http(r) if r.status() == StatusCode::UNAUTHORIZED => Fail::Locked,
                e => Fail::Off(e.to_string()),
            })?;
        Ok(s)
    }

    fn auth(&self, l: &Link, r: RequestBuilder) -> RequestBuilder {
        if l.token.is_empty() { r } else { r.bearer_auth(&l.token) }
    }
}

fn tls() -> Arc<rustls::ClientConfig> {
    // Vale para o processo todo: outro cliente rustls que apareça usa o mesmo ring.
    let _ = rustls::crypto::ring::default_provider().install_default();
    let roots = rustls::RootCertStore { roots: webpki_roots::TLS_SERVER_ROOTS.to_vec() };
    let cfg = rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions()
        .expect("tls")
        .with_root_certificates(roots)
        .with_no_client_auth();
    Arc::new(cfg)
}

/// Rotas de sync: 404 quer dizer que o outro lado ainda não tem sync.
async fn sync(r: RequestBuilder) -> Result<Response, Fail> {
    let r = r.send().await.map_err(off)?;
    if r.status() == StatusCode::NOT_FOUND {
        return Err(Fail::Old);
    }
    ok(r).await
}

async fn ok(r: Response) -> Result<Response, Fail> {
    match r.status() {
        s if s.is_success() => Ok(r),
        StatusCode::UNAUTHORIZED => Err(Fail::Locked),
        s => {
            let msg = r.json::<serde_json::Value>().await.ok().and_then(|v| v["error"].as_str().map(Into::into));
            Err(Fail::Bad(msg.unwrap_or_else(|| s.to_string())))
        }
    }
}

fn off(e: reqwest::Error) -> Fail {
    Fail::Off(e.without_url().to_string())
}
