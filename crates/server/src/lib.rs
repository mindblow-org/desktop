mod config;
mod http;
mod live;
mod web;

use std::sync::Arc;

use domain::{Blobs, Bus, Notes, Repo};

pub use config::Config;
pub use http::router;
pub use live::Live;

#[derive(Clone)]
pub struct App {
    pub notes: Arc<Notes>,
    pub live: Arc<Live>,
    pub token: Option<Arc<str>>,
    pub hosts: Arc<[String]>,
    pub link: Option<Arc<str>>,
}

impl App {
    pub fn new(repo: Arc<dyn Repo>, blobs: Arc<dyn Blobs>, cfg: &Config) -> Self {
        let bus = Arc::new(Bus::default());
        let live = Arc::new(Live::new());
        bus.join(live.clone());
        Self {
            notes: Arc::new(Notes::new(repo, blobs, bus)),
            live,
            token: cfg.token.as_deref().map(Into::into),
            hosts: cfg.hosts.clone().into(),
            link: cfg.link.as_deref().map(Into::into),
        }
    }
}
