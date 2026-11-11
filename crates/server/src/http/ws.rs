use std::sync::Arc;

use axum::{
    extract::{
        Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use domain::Id;
use serde::Deserialize;
use tokio::sync::broadcast::error::RecvError;

use crate::{App, Live};

const RESYNC: &str = r#"{"type":"resync"}"#;

#[derive(Deserialize)]
pub struct Dev {
    d: Option<String>,
    n: Option<String>,
}

#[derive(Deserialize)]
struct Say {
    r#type: String,
    name: String,
}

pub async fn live(ws: WebSocketUpgrade, Query(q): Query<Dev>, State(a): State<App>) -> Response {
    let dev = q.d.filter(|d| !d.is_empty() && d.len() <= 64).unwrap_or_else(|| Id::new().to_string());
    let name = q.n.unwrap_or_default();
    ws.on_upgrade(move |s| run(s, a.live, dev, name))
}

async fn run(mut s: WebSocket, live: Arc<Live>, dev: String, name: String) {
    let mut rx = live.rx();
    live.join(&dev, &name);
    loop {
        tokio::select! {
            m = rx.recv() => {
                let t = match m {
                    Ok(t) => t,
                    Err(RecvError::Lagged(_)) => RESYNC.into(),
                    Err(RecvError::Closed) => break,
                };
                if s.send(Message::Text(t.as_ref().into())).await.is_err() {
                    break;
                }
            }
            m = s.recv() => match m {
                Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                Some(Ok(Message::Text(t))) => {
                    if let Ok(m) = serde_json::from_str::<Say>(&t) && m.r#type == "name" {
                        live.rename(&dev, &m.name);
                    }
                }
                _ => {}
            }
        }
    }
    live.leave(&dev);
}
