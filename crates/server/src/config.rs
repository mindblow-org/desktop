use std::{
    env::var,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

pub const PORT: u16 = 7341;

#[derive(Clone, Debug)]
pub struct Config {
    pub addr: SocketAddr,
    pub db: PathBuf,
    pub token: Option<String>,
    pub hosts: Vec<String>,
    pub link: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), PORT),
            db: data().join("mindblow.db"),
            token: None,
            hosts: Vec::new(),
            link: None,
        }
    }
}

impl Config {
    pub fn env() -> Self {
        let d = Self::default();
        let ip = var("MINDBLOW_HOST").ok().and_then(|v| v.parse().ok()).unwrap_or(d.addr.ip());
        let port = var("MINDBLOW_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(PORT);
        Self {
            addr: SocketAddr::new(ip, port),
            db: var("MINDBLOW_DB").map(PathBuf::from).unwrap_or(d.db),
            token: var("MINDBLOW_TOKEN").ok().filter(|t| !t.is_empty()),
            hosts: var("MINDBLOW_HOSTS")
                .map(|v| v.split(',').map(|h| h.trim().to_ascii_lowercase()).filter(|h| !h.is_empty()).collect())
                .unwrap_or_default(),
            link: var("MINDBLOW_LINK").ok().map(|v| v.trim().trim_end_matches('/').to_owned()).filter(|v| !v.is_empty()),
        }
    }
}

fn data() -> PathBuf {
    var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|_| var("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("mindblow")
}
