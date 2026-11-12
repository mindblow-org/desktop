use std::{fmt::Display, path::Path, sync::Mutex};

use domain::{Blobs, Change, Error, Gone, Id, Link, Links, Note, Repo, Result};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

const TUNE: &str = "pragma journal_mode = wal; pragma synchronous = normal;";
const STEPS: &[&str] = &[
    include_str!("sql/1-notes.sql"),
    include_str!("sql/2-pinned.sql"),
    include_str!("sql/3-images.sql"),
    include_str!("sql/4-sync.sql"),
    include_str!("sql/5-color.sql"),
];
const COLS: &str = "id, kind, title, body, items, images, pinned, rank, created, updated, clock, color";

pub struct Sqlite {
    db: Mutex<Connection>,
}

impl Sqlite {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let p = path.as_ref();
        if let Some(d) = p.parent() {
            std::fs::create_dir_all(d).map_err(fail)?;
        }
        Self::with(Connection::open(p).map_err(fail)?)
    }

    pub fn memory() -> Result<Self> {
        Self::with(Connection::open_in_memory().map_err(fail)?)
    }

    fn with(mut c: Connection) -> Result<Self> {
        c.execute_batch(TUNE).map_err(fail)?;
        migrate(&mut c).map_err(fail)?;
        c.execute("insert or ignore into meta (k, v) values ('node', ?1)", [Id::new().to_string()])
            .map_err(fail)?;
        Ok(Self { db: Mutex::new(c) })
    }

    fn write<T>(&self, f: impl FnOnce(&Transaction) -> rusqlite::Result<T>) -> Result<T> {
        let mut db = self.db.lock().unwrap();
        let t = db.transaction().map_err(fail)?;
        let v = f(&t).map_err(fail)?;
        t.commit().map_err(fail)?;
        Ok(v)
    }
}

/// Próxima posição do feed.
fn bump(t: &Transaction) -> rusqlite::Result<i64> {
    t.query_row("update meta set v = v + 1 where k = 'seq' returning v", [], |r| r.get(0))
}

impl Repo for Sqlite {
    fn all(&self) -> Result<Vec<Note>> {
        let db = self.db.lock().unwrap();
        let mut q = db.prepare(&format!("select {COLS} from notes order by rank")).map_err(fail)?;
        let rows = q.query_map([], raw).map_err(fail)?;
        rows.map(|r| r.map_err(fail)?.note()).collect()
    }

    fn get(&self, id: Id) -> Result<Option<Note>> {
        let db = self.db.lock().unwrap();
        db.query_row(&format!("select {COLS} from notes where id = ?1"), [id.to_string()], raw)
            .optional()
            .map_err(fail)?
            .map(Raw::note)
            .transpose()
    }

    fn put(&self, n: &Note) -> Result<()> {
        let items = serde_json::to_string(&n.items).map_err(fail)?;
        let images = serde_json::to_string(&n.images).map_err(fail)?;
        let clock = serde_json::to_string(&n.clock).map_err(fail)?;
        let id = n.id.to_string();
        self.write(|t| {
            let seq = bump(t)?;
            t.execute(
                &format!(
                    "insert into notes ({COLS}, seq) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                     on conflict(id) do update set kind = excluded.kind, title = excluded.title,
                     body = excluded.body, items = excluded.items, images = excluded.images,
                     pinned = excluded.pinned, rank = excluded.rank, created = excluded.created,
                     updated = excluded.updated, clock = excluded.clock, color = excluded.color, seq = excluded.seq"
                ),
                params![id, n.kind.name(), n.title, n.body, items, images, n.pinned, n.rank, n.created, n.updated, clock, n.color.name(), seq],
            )?;
            t.execute("delete from gone where id = ?1", [&id])?;
            Ok(())
        })
    }

    fn del(&self, id: Id, at: i64) -> Result<bool> {
        let id = id.to_string();
        self.write(|t| {
            if t.execute("delete from notes where id = ?1", [&id])? == 0 {
                return Ok(false);
            }
            let seq = bump(t)?;
            t.execute("insert or replace into gone (id, at, seq) values (?1, ?2, ?3)", params![id, at, seq])?;
            Ok(true)
        })
    }

    fn bury(&self, id: Id, at: i64) -> Result<()> {
        let id = id.to_string();
        self.write(|t| {
            let had: Option<i64> = t.query_row("select at from gone where id = ?1", [&id], |r| r.get(0)).optional()?;
            if had.is_some_and(|h| h >= at) {
                return Ok(());
            }
            let seq = bump(t)?;
            t.execute("insert or replace into gone (id, at, seq) values (?1, ?2, ?3)", params![id, at, seq])?;
            Ok(())
        })
    }

    fn gone(&self, id: Id) -> Result<Option<i64>> {
        self.db
            .lock()
            .unwrap()
            .query_row("select at from gone where id = ?1", [id.to_string()], |r| r.get(0))
            .optional()
            .map_err(fail)
    }

    fn head(&self) -> Result<Option<f64>> {
        self.db
            .lock()
            .unwrap()
            .query_row("select min(rank) from notes", [], |r| r.get(0))
            .map_err(fail)
    }

    fn seq(&self) -> Result<i64> {
        self.db
            .lock()
            .unwrap()
            .query_row("select v from meta where k = 'seq'", [], |r| r.get(0))
            .map_err(fail)
    }

    fn feed(&self, since: i64, limit: usize) -> Result<Vec<(i64, Change)>> {
        let db = self.db.lock().unwrap();
        let mut v = Vec::new();
        let mut q = db
            .prepare(&format!("select {COLS}, seq from notes where seq > ?1 order by seq limit ?2"))
            .map_err(fail)?;
        let rows = q
            .query_map(params![since, limit as i64], |r| Ok((r.get::<_, i64>(12)?, raw(r)?)))
            .map_err(fail)?;
        for r in rows {
            let (seq, n) = r.map_err(fail)?;
            v.push((seq, Change::Put(n.note()?)));
        }
        let mut q = db.prepare("select id, at, seq from gone where seq > ?1 order by seq limit ?2").map_err(fail)?;
        let rows = q
            .query_map(params![since, limit as i64], |r| Ok((r.get::<_, String>(0)?, r.get(1)?, r.get(2)?)))
            .map_err(fail)?;
        for r in rows {
            let (id, at, seq) = r.map_err(fail)?;
            v.push((seq, Change::Gone(Gone { id: id.parse()?, at })));
        }
        v.sort_by_key(|(s, _)| *s);
        v.truncate(limit);
        Ok(v)
    }
}

impl Blobs for Sqlite {
    fn put(&self, id: Id, mime: &str, data: &[u8]) -> Result<()> {
        self.db
            .lock()
            .unwrap()
            .execute("insert or replace into blobs (id, mime, data) values (?1, ?2, ?3)", params![id.to_string(), mime, data])
            .map_err(fail)?;
        Ok(())
    }

    fn get(&self, id: Id) -> Result<Option<(String, Vec<u8>)>> {
        self.db
            .lock()
            .unwrap()
            .query_row("select mime, data from blobs where id = ?1", [id.to_string()], |r| Ok((r.get(0)?, r.get(1)?)))
            .optional()
            .map_err(fail)
    }

    fn del(&self, id: Id) -> Result<()> {
        self.db.lock().unwrap().execute("delete from blobs where id = ?1", [id.to_string()]).map_err(fail)?;
        Ok(())
    }

    fn has(&self, id: Id) -> Result<bool> {
        self.db
            .lock()
            .unwrap()
            .query_row("select 1 from blobs where id = ?1", [id.to_string()], |_| Ok(()))
            .optional()
            .map(|r| r.is_some())
            .map_err(fail)
    }
}

impl Links for Sqlite {
    fn node(&self) -> Result<Id> {
        let v: String = self
            .db
            .lock()
            .unwrap()
            .query_row("select v from meta where k = 'node'", [], |r| r.get(0))
            .map_err(fail)?;
        v.parse()
    }

    fn links(&self) -> Result<Vec<Link>> {
        let db = self.db.lock().unwrap();
        let mut q = db.prepare("select url, token, pull, push from links order by url").map_err(fail)?;
        let rows = q
            .query_map([], |r| Ok(Link { url: r.get(0)?, token: r.get(1)?, pull: r.get(2)?, push: r.get(3)? }))
            .map_err(fail)?;
        rows.map(|r| r.map_err(fail)).collect()
    }

    fn join(&self, l: &Link) -> Result<()> {
        self.db
            .lock()
            .unwrap()
            .execute(
                "insert into links (url, token, pull, push) values (?1, ?2, ?3, ?4)
                 on conflict(url) do update set token = excluded.token",
                params![l.url, l.token, l.pull, l.push],
            )
            .map_err(fail)?;
        Ok(())
    }

    fn part(&self, url: &str) -> Result<bool> {
        let n = self.db.lock().unwrap().execute("delete from links where url = ?1", [url]).map_err(fail)?;
        Ok(n > 0)
    }

    fn moved(&self, url: &str, pull: i64, push: i64) -> Result<()> {
        self.db
            .lock()
            .unwrap()
            .execute("update links set pull = ?2, push = ?3 where url = ?1", params![url, pull, push])
            .map_err(fail)?;
        Ok(())
    }
}

struct Raw {
    id: String,
    kind: String,
    title: String,
    body: String,
    items: String,
    images: String,
    pinned: bool,
    rank: f64,
    created: i64,
    updated: i64,
    clock: String,
    color: String,
}

impl Raw {
    fn note(self) -> Result<Note> {
        Ok(Note {
            id: self.id.parse()?,
            kind: self.kind.parse()?,
            title: self.title,
            body: self.body,
            items: serde_json::from_str(&self.items).map_err(fail)?,
            images: serde_json::from_str(&self.images).map_err(fail)?,
            pinned: self.pinned,
            color: self.color.parse()?,
            rank: self.rank,
            created: self.created,
            updated: self.updated,
            clock: serde_json::from_str(&self.clock).map_err(fail)?,
        })
    }
}

fn raw(r: &Row) -> rusqlite::Result<Raw> {
    Ok(Raw {
        id: r.get(0)?,
        kind: r.get(1)?,
        title: r.get(2)?,
        body: r.get(3)?,
        items: r.get(4)?,
        images: r.get(5)?,
        pinned: r.get(6)?,
        rank: r.get(7)?,
        created: r.get(8)?,
        updated: r.get(9)?,
        clock: r.get(10)?,
        color: r.get(11)?,
    })
}

fn migrate(c: &mut Connection) -> rusqlite::Result<()> {
    let done: i64 = c.query_row("pragma user_version", [], |r| r.get(0))?;
    for (i, step) in (1..).zip(STEPS).skip(done as usize) {
        let t = c.transaction()?;
        t.execute_batch(step)?;
        t.pragma_update(None, "user_version", i)?;
        t.commit()?;
    }
    Ok(())
}

fn fail(e: impl Display) -> Error {
    Error::Store(e.to_string())
}
