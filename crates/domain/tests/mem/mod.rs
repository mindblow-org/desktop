use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use domain::{Blobs, Bus, Change, Event, Gone, Id, Note, Notes, Repo, Result};

/// Repo em memória com o feed de mudanças, como o SQLite.
#[derive(Default)]
pub struct Mem(Mutex<State>);

#[derive(Default)]
pub struct State {
    seq: i64,
    notes: HashMap<Id, (i64, Note)>,
    gone: HashMap<Id, (i64, i64)>,
}

impl Repo for Mem {
    fn all(&self) -> Result<Vec<Note>> {
        Ok(self.0.lock().unwrap().notes.values().map(|(_, n)| n.clone()).collect())
    }

    fn get(&self, id: Id) -> Result<Option<Note>> {
        Ok(self.0.lock().unwrap().notes.get(&id).map(|(_, n)| n.clone()))
    }

    fn put(&self, n: &Note) -> Result<()> {
        let mut s = self.0.lock().unwrap();
        s.seq += 1;
        let seq = s.seq;
        s.gone.remove(&n.id);
        s.notes.insert(n.id, (seq, n.clone()));
        Ok(())
    }

    fn del(&self, id: Id, at: i64) -> Result<bool> {
        let mut s = self.0.lock().unwrap();
        if s.notes.remove(&id).is_none() {
            return Ok(false);
        }
        s.seq += 1;
        let seq = s.seq;
        s.gone.insert(id, (seq, at));
        Ok(true)
    }

    fn bury(&self, id: Id, at: i64) -> Result<()> {
        let mut s = self.0.lock().unwrap();
        if s.gone.get(&id).is_some_and(|(_, a)| *a >= at) {
            return Ok(());
        }
        s.seq += 1;
        let seq = s.seq;
        s.gone.insert(id, (seq, at));
        Ok(())
    }

    fn gone(&self, id: Id) -> Result<Option<i64>> {
        Ok(self.0.lock().unwrap().gone.get(&id).map(|(_, at)| *at))
    }

    fn head(&self) -> Result<Option<f64>> {
        Ok(self.0.lock().unwrap().notes.values().map(|(_, n)| n.rank).reduce(f64::min))
    }

    fn seq(&self) -> Result<i64> {
        Ok(self.0.lock().unwrap().seq)
    }

    fn feed(&self, since: i64, limit: usize) -> Result<Vec<(i64, Change)>> {
        let s = self.0.lock().unwrap();
        let mut v: Vec<_> = s
            .notes
            .values()
            .filter(|(q, _)| *q > since)
            .map(|(q, n)| (*q, Change::Put(n.clone())))
            .chain(s.gone.iter().filter(|(_, (q, _))| *q > since).map(|(id, (q, at))| (*q, Change::Gone(Gone { id: *id, at: *at }))))
            .collect();
        v.sort_by_key(|(q, _)| *q);
        v.truncate(limit);
        Ok(v)
    }
}

#[derive(Default)]
pub struct Files(pub Mutex<HashMap<Id, (String, Vec<u8>)>>);

impl Blobs for Files {
    fn put(&self, id: Id, mime: &str, data: &[u8]) -> Result<()> {
        self.0.lock().unwrap().insert(id, (mime.into(), data.into()));
        Ok(())
    }

    fn get(&self, id: Id) -> Result<Option<(String, Vec<u8>)>> {
        Ok(self.0.lock().unwrap().get(&id).cloned())
    }

    fn del(&self, id: Id) -> Result<()> {
        self.0.lock().unwrap().remove(&id);
        Ok(())
    }
}

pub type Log = Arc<Mutex<Vec<Event>>>;

pub fn full() -> (Notes, Log, Arc<Files>) {
    let bus = Arc::new(Bus::default());
    let log: Log = Arc::default();
    let l = log.clone();
    bus.join(Arc::new(move |e: &Event| l.lock().unwrap().push(e.clone())));
    let files = Arc::new(Files::default());
    (Notes::new(Arc::new(Mem::default()), files.clone(), bus), log, files)
}
