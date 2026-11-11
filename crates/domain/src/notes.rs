use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{Blobs, Bus, Draft, Error, Event, Id, Image, Item, Kind, Note, Patch, Repo, Result, item, rank, search};

pub struct Notes {
    repo: Arc<dyn Repo>,
    blobs: Arc<dyn Blobs>,
    bus: Arc<Bus>,
    lock: Mutex<()>,
}

impl Notes {
    pub fn new(repo: Arc<dyn Repo>, blobs: Arc<dyn Blobs>, bus: Arc<Bus>) -> Self {
        Self { repo, blobs, bus, lock: Mutex::new(()) }
    }

    pub fn bus(&self) -> &Arc<Bus> {
        &self.bus
    }

    pub fn list(&self) -> Result<Vec<Note>> {
        let mut v = self.repo.all()?;
        v.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(a.rank.total_cmp(&b.rank)).then(a.id.cmp(&b.id)));
        Ok(v)
    }

    pub fn find(&self, q: &str) -> Result<Vec<Note>> {
        let mut v = self.list()?;
        v.retain(|n| search::hit(n, q));
        Ok(v)
    }

    pub fn get(&self, id: Id) -> Result<Note> {
        self.repo.get(id)?.ok_or(Error::Missing)
    }

    pub fn add(&self, d: Draft) -> Result<Note> {
        let _g = self.lock.lock().unwrap();
        let n = d.note(rank::between(None, self.repo.head()?), now());
        n.check()?;
        self.repo.put(&n)?;
        self.bus.emit(Event::Created { note: n.clone() });
        Ok(n)
    }

    pub fn edit(&self, id: Id, p: Patch) -> Result<Note> {
        self.change(id, |n| {
            p.apply(n);
            Ok(())
        })
    }

    pub fn remove(&self, id: Id) -> Result<()> {
        let _g = self.lock.lock().unwrap();
        let n = self.get(id)?;
        if !self.repo.del(id)? {
            return Err(Error::Missing);
        }
        for i in &n.images {
            self.blobs.del(i.id)?;
        }
        self.bus.emit(Event::Deleted { id });
        Ok(())
    }

    pub fn attach(&self, id: Id, mime: &str, data: &[u8], w: u32, h: u32) -> Result<Note> {
        let img = Image::new(mime, data.len(), w, h)?;
        let key = img.id;
        self.change(id, |n| {
            n.images.push(img);
            n.check()?;
            self.blobs.put(key, mime, data)
        })
    }

    pub fn detach(&self, id: Id, img: Id) -> Result<Note> {
        let n = self.change(id, |n| {
            let len = n.images.len();
            n.images.retain(|i| i.id != img);
            if n.images.len() == len { Err(Error::NoImage) } else { Ok(()) }
        })?;
        self.blobs.del(img)?;
        Ok(n)
    }

    pub fn image(&self, id: Id) -> Result<(String, Vec<u8>)> {
        self.blobs.get(id)?.ok_or(Error::NoImage)
    }

    pub fn push(&self, id: Id, i: Item) -> Result<Note> {
        self.change(id, |n| {
            if n.kind != Kind::Todo {
                return Err(Error::Invalid("só todo tem itens"));
            }
            if n.items.iter().any(|x| x.id == i.id) {
                return Err(Error::Invalid("item repetido"));
            }
            n.items.push(i);
            Ok(())
        })
    }

    pub fn mark(&self, id: Id, item: Id, p: item::Patch) -> Result<Note> {
        self.change(id, |n| {
            p.apply(n.item(item)?);
            Ok(())
        })
    }

    pub fn pull(&self, id: Id, item: Id) -> Result<Note> {
        self.change(id, |n| {
            let len = n.items.len();
            n.items.retain(|i| i.id != item);
            if n.items.len() == len { Err(Error::NoItem) } else { Ok(()) }
        })
    }

    fn change(&self, id: Id, f: impl FnOnce(&mut Note) -> Result<()>) -> Result<Note> {
        let _g = self.lock.lock().unwrap();
        let mut n = self.get(id)?;
        f(&mut n)?;
        if !n.rank.is_finite() {
            return Err(Error::Invalid("rank inválido"));
        }
        n.check()?;
        n.updated = now().max(n.updated + 1);
        self.repo.put(&n)?;
        self.bus.emit(Event::Updated { note: n.clone() });
        Ok(n)
    }
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
