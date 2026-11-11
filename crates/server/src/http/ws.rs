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
}

pub async fn live(ws: WebSocketUpgrade, Query(q): Query<Dev>, State(a): State<App>) -> Response {
    let dev = q.d.filter(|d| !d.is_empty() && d.len() <= 64).unwrap_or_else(|| Id::new().to_string());
    ws.on_upgrade(move |s| run(s, a.live, dev))
}

async fn run(mut s: WebSocket, live: Arc<Live>, dev: String) {
    let mut rx = live.rx();
    live.join(&dev);
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
                _ => {}
            }
        }
    }
    live.leave(&dev);
}
