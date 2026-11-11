use axum::{
    extract::{Request, State},
    http::{
        HeaderMap, StatusCode,
        header::{AUTHORIZATION, HOST, ORIGIN},
    },
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::App;

pub async fn check(State(a): State<App>, req: Request, next: Next) -> Response {
    let h = req.headers();
    let host = h.get(HOST).and_then(|v| v.to_str().ok()).or(req.uri().host()).unwrap_or_default();
    if !allowed(bare(host), &a.hosts) {
        return (StatusCode::FORBIDDEN, "host não permitido").into_response();
    }
    if h.get(ORIGIN).and_then(|v| v.to_str().ok()).is_some_and(|o| !same_site(o, host)) {
        return (StatusCode::FORBIDDEN, "origem não permitida").into_response();
    }
    let api = req.uri().path().starts_with("/api/");
    if a.token.as_deref().is_some_and(|t| api && !token(h, req.uri().query(), t)) {
        return (StatusCode::UNAUTHORIZED, "token inválido").into_response();
    }
    next.run(req).await
}

fn same_site(origin: &str, host: &str) -> bool {
    let Some((scheme, auth)) = origin.split_once("://") else { return false };
    let fallback = if scheme == "https" { 443 } else { 80 };
    let (on, op) = (bare(auth), port(auth).unwrap_or(fallback));
    on.eq_ignore_ascii_case(bare(host)) && port(host).is_none_or(|p| p == op)
}

fn port(h: &str) -> Option<u16> {
    let rest = match h.strip_prefix('[') {
        Some(r) => r.split_once(']')?.1,
        None => h,
    };
    rest.rsplit_once(':')?.1.parse().ok()
}

fn bare(h: &str) -> &str {
    match h.strip_prefix('[') {
        Some(r) => r.split(']').next().unwrap_or(r),
        None => h.rsplit_once(':').map_or(h, |(a, _)| a),
    }
}

fn allowed(h: &str, extra: &[String]) -> bool {
    let h = h.to_ascii_lowercase();
    matches!(h.as_str(), "localhost" | "127.0.0.1" | "::1")
        || h.ends_with(".ts.net")
        || (!h.is_empty() && !h.contains('.') && !h.contains(':'))
        || extra.contains(&h)
}

fn token(h: &HeaderMap, q: Option<&str>, want: &str) -> bool {
    let bearer = h
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    let query = q.and_then(|q| q.split('&').find_map(|kv| kv.strip_prefix("token=")));
    bearer.or(query).is_some_and(|g| same(g.as_bytes(), want.as_bytes()))
}

fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0, |acc, (x, y)| acc | (x ^ y)) == 0
}
