use std::{
    collections::HashSet,
    sync::{
        Arc, Mutex,
        atomic::{AtomicI64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    Blobs, Bus, Change, Draft, Error, Event, Feed, Gone, Id, Image, Item, Kind, Note, Patch, Repo, Result, Shot, Took, item,
    merge, rank, search,
};

/// Carimbo de outro aparelho mais adiantado que isso não puxa o relógio daqui.
const AHEAD: i64 = 24 * 60 * 60 * 1000;

pub struct Notes {
    repo: Arc<dyn Repo>,
    blobs: Arc<dyn Blobs>,
    bus: Arc<Bus>,
    lock: Mutex<()>,
    last: AtomicI64,
}

impl Notes {
    pub fn new(repo: Arc<dyn Repo>, blobs: Arc<dyn Blobs>, bus: Arc<Bus>) -> Self {
        Self { repo, blobs, bus, lock: Mutex::new(()), last: AtomicI64::new(0) }
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
        let n = d.note(rank::between(None, self.repo.head()?), self.tick(0));
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
        if !self.repo.del(id, self.tick(n.updated))? {
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

    /// Até `limit` mudanças deste aparelho depois de `since`, para outro aparelho levar.
    pub fn feed(&self, since: i64, limit: usize) -> Result<Feed> {
        let v = self.repo.feed(since, limit)?;
        let mut f = Feed { seq: since, more: v.len() >= limit, ..Feed::default() };
        for (seq, c) in v {
            f.seq = seq;
            match c {
                Change::Put(n) => f.notes.push(n.into()),
                Change::Gone(g) => f.gone.push(g),
            }
        }
        Ok(f)
    }

    pub fn seq(&self) -> Result<i64> {
        self.repo.seq()
    }

    /// Junta as mudanças que vieram de outro aparelho. Só grava (e avisa) o que muda algo
    /// aqui, então o que volta de onde saiu não ecoa. Nota excluída lá e editada aqui
    /// depois da exclusão fica, e a edição volta para lá.
    pub fn take(&self, notes: Vec<Shot>, gone: Vec<Gone>) -> Result<Took> {
        let _g = self.lock.lock().unwrap();
        let from = self.repo.seq()?;
        let mut need = Vec::new();
        for s in notes {
            let n = Note::from(s);
            if n.check().is_err() || !n.rank.is_finite() {
                continue;
            }
            self.seen(n.updated);
            if let Some(m) = self.meet(n)? {
                for i in &m.images {
                    if !self.blobs.has(i.id)? {
                        need.push(i.id);
                    }
                }
            }
        }
        for g in gone {
            self.seen(g.at);
            match self.repo.get(g.id)? {
                Some(n) if n.updated <= g.at => {
                    self.repo.del(g.id, g.at)?;
                    for i in &n.images {
                        self.blobs.del(i.id)?;
                    }
                    self.bus.emit(Event::Deleted { id: g.id });
                }
                Some(_) => {}
                None => self.repo.bury(g.id, g.at)?,
            }
        }
        Ok(Took { from, to: self.repo.seq()?, need })
    }

    /// Grava a versão que veio junto com a daqui. Devolve a nota se ela mudou.
    fn meet(&self, n: Note) -> Result<Option<Note>> {
        let Some(old) = self.repo.get(n.id)? else {
            if self.repo.gone(n.id)?.is_some_and(|at| at >= n.updated) {
                return Ok(None);
            }
            self.repo.put(&n)?;
            self.bus.emit(Event::Created { note: n.clone() });
            return Ok(Some(n));
        };
        let m = merge(&old, &n);
        if m == old {
            return Ok(None);
        }
        self.repo.put(&m)?;
        let keep: HashSet<Id> = m.images.iter().map(|i| i.id).collect();
        for i in old.images.iter().filter(|i| !keep.contains(&i.id)) {
            self.blobs.del(i.id)?;
        }
        self.bus.emit(Event::Updated { note: m.clone() });
        Ok(Some(m))
    }

    /// Guarda os bytes de uma imagem que veio de outro aparelho, com o id que ela já tem.
    pub fn adopt(&self, id: Id, mime: &str, data: &[u8]) -> Result<()> {
        Image::new(mime, data.len(), 0, 0)?;
        self.blobs.put(id, mime, data)
    }

    /// Imagens que alguma nota usa e que ainda não chegaram.
    pub fn lack(&self) -> Result<Vec<Id>> {
        let mut v = Vec::new();
        for n in self.repo.all()? {
            for i in n.images {
                if !self.blobs.has(i.id)? {
                    v.push(i.id);
                }
            }
        }
        Ok(v)
    }

    fn change(&self, id: Id, f: impl FnOnce(&mut Note) -> Result<()>) -> Result<Note> {
        let _g = self.lock.lock().unwrap();
        let old = self.get(id)?;
        let mut n = old.clone();
        f(&mut n)?;
        if !n.rank.is_finite() {
            return Err(Error::Invalid("rank inválido"));
        }
        n.check()?;
        n.stamp(&old, self.tick(old.updated));
        self.repo.put(&n)?;
        self.bus.emit(Event::Updated { note: n.clone() });
        Ok(n)
    }

    /// Relógio híbrido: nunca anda para trás e passa de tudo que já viu, então uma
    /// edição feita depois de receber outra ganha dela mesmo com o relógio atrasado.
    fn tick(&self, after: i64) -> i64 {
        let t = now().max(self.last.load(Ordering::Relaxed) + 1).max(after + 1);
        self.last.fetch_max(t, Ordering::Relaxed);
        t
    }

    fn seen(&self, t: i64) {
        if t <= now() + AHEAD {
            self.last.fetch_max(t, Ordering::Relaxed);
        }
    }
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
