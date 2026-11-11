use std::process::exit;

use server::{App, Config, serve};
use tokio::{net::TcpListener, signal};

#[tokio::main]
async fn main() {
    let cfg = Config::env();
    let app = App::open(&cfg).unwrap_or_else(|e| die(e));
    let l = TcpListener::bind(cfg.addr).await.unwrap_or_else(|e| die(e));
    eprintln!("mindblow em http://{} · {}", cfg.addr, cfg.db.display());
    if let Err(e) = serve(app, &cfg, l, stop()).await {
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
