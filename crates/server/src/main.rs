use std::{process::exit, sync::Arc};

use server::{App, Config, router};
use store::Sqlite;
use tokio::{net::TcpListener, signal};

#[tokio::main]
async fn main() {
    let cfg = Config::env();
    let db = Arc::new(Sqlite::open(&cfg.db).unwrap_or_else(|e| die(e)));
    let app = App::new(db.clone(), db, &cfg);
    let l = TcpListener::bind(cfg.addr).await.unwrap_or_else(|e| die(e));
    eprintln!("mindblow em http://{} · {}", cfg.addr, cfg.db.display());
    if let Err(e) = axum::serve(l, router(app)).with_graceful_shutdown(stop()).await {
        die(e);
    }
}

async fn stop() {
    let int = signal::ctrl_c();
    #[cfg(unix)]
    {
        let mut term = signal::unix::signal(signal::unix::SignalKind::terminate()).expect("sigterm");
        tokio::select! {
            _ = int => {}
            _ = term.recv() => {}
        }
    }
    #[cfg(not(unix))]
    let _ = int.await;
}

fn die(e: impl std::fmt::Display) -> ! {
    eprintln!("mindblow: {e}");
    exit(1)
}
