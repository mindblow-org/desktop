use std::{fmt::Display, path::Path, sync::Mutex};

use domain::{Blobs, Error, Id, Note, Repo, Result};
use rusqlite::{Connection, OptionalExtension, Row, params};

const TUNE: &str = "pragma journal_mode = wal; pragma synchronous = normal;";
const STEPS: &[&str] = &[
    include_str!("sql/1-notes.sql"),
    include_str!("sql/2-pinned.sql"),
    include_str!("sql/3-images.sql"),
];
const COLS: &str = "id, kind, title, body, items, images, pinned, rank, created, updated";

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
        Ok(Self { db: Mutex::new(c) })
    }
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
        self.db
            .lock()
            .unwrap()
            .execute(
                &format!(
                    "insert into notes ({COLS}) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                     on conflict(id) do update set kind = excluded.kind, title = excluded.title,
                     body = excluded.body, items = excluded.items, images = excluded.images,
                     pinned = excluded.pinned, rank = excluded.rank, updated = excluded.updated"
                ),
                params![n.id.to_string(), n.kind.name(), n.title, n.body, items, images, n.pinned, n.rank, n.created, n.updated],
            )
            .map_err(fail)?;
        Ok(())
    }

    fn del(&self, id: Id) -> Result<bool> {
        let n = self
            .db
            .lock()
            .unwrap()
            .execute("delete from notes where id = ?1", [id.to_string()])
            .map_err(fail)?;
        Ok(n > 0)
    }

    fn head(&self) -> Result<Option<f64>> {
        self.db
            .lock()
            .unwrap()
            .query_row("select min(rank) from notes", [], |r| r.get(0))
            .map_err(fail)
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
            rank: self.rank,
            created: self.created,
            updated: self.updated,
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
