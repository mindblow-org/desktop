mod config;
mod http;
mod live;
pub mod sync;
mod web;

use std::{future::Future, io, net::SocketAddr, sync::Arc};

use domain::{Blobs, Bus, Id, Links, Notes, Repo};
use store::Sqlite;
use tokio::net::TcpListener;

pub use config::Config;
pub use http::router;
pub use live::{Live, Peer};
pub use sync::Syncer;

#[derive(Clone)]
pub struct App {
    pub notes: Arc<Notes>,
    pub live: Arc<Live>,
    pub sync: Arc<Syncer>,
    pub node: Id,
    pub token: Option<Arc<str>>,
    pub hosts: Arc<[String]>,
    pub link: Option<Arc<str>>,
}

impl App {
    pub fn new(repo: Arc<dyn Repo>, blobs: Arc<dyn Blobs>, links: Arc<dyn Links>, cfg: &Config) -> domain::Result<Self> {
        let node = links.node()?;
        let bus = Arc::new(Bus::default());
        let live = Arc::new(Live::new(&node.to_string()));
        let notes = Arc::new(Notes::new(repo, blobs, bus.clone()));
        let sync = Arc::new(Syncer::new(notes.clone(), links, live.clone(), node));
        bus.join(live.clone());
        bus.join(sync.clone());
        Ok(Self {
            notes,
            live,
            sync,
            node,
            token: cfg.token.as_deref().map(Into::into),
            hosts: cfg.hosts.clone().into(),
            link: cfg.link.as_deref().map(Into::into),
        })
    }

    /// Abre o banco de `cfg.db`.
    pub fn open(cfg: &Config) -> domain::Result<Self> {
        let db = Arc::new(Sqlite::open(&cfg.db)?);
        Self::new(db.clone(), db.clone(), db, cfg)
    }
}

/// Liga o sync (com o link de `cfg.peer`, se houver) e atende em `l` até `stop`.
pub async fn serve(app: App, cfg: &Config, l: TcpListener, stop: impl Future<Output = ()> + Send + 'static) -> io::Result<()> {
    if let Some((url, token)) = cfg.peer.as_deref().and_then(sync::parse) {
        app.sync.add(&url, &token).map_err(io::Error::other)?;
    }
    app.sync.start().map_err(io::Error::other)?;
    axum::serve(l, router(app)).with_graceful_shutdown(stop).await
}

/// Sobe o mindblow numa thread própria, para o app desktop e o Android levarem o
/// servidor junto. Devolve o endereço assim que a porta está ocupada; porta já em uso
/// volta como erro `AddrInUse`.
pub fn spawn(cfg: Config) -> io::Result<SocketAddr> {
    let l = std::net::TcpListener::bind(cfg.addr)?;
    l.set_nonblocking(true)?;
    let addr = l.local_addr()?;
    let app = App::open(&cfg).map_err(io::Error::other)?;
    let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build()?;
    std::thread::Builder::new().name("mindblow".into()).spawn(move || {
        rt.block_on(async move {
            let l = TcpListener::from_std(l).expect("porta");
            if let Err(e) = serve(app, &cfg, l, std::future::pending()).await {
                eprintln!("mindblow: {e}");
            }
        })
    })?;
    Ok(addr)
}
